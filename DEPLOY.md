# 部署指南 — 瑜安伽 Yuan Yoga

把這個 app 免費部署到網路上,讓別人可以用瀏覽器打開。

## 為什麼不能用 GitHub Pages

GitHub Pages 只能放**靜態網頁**。這個 app 需要一個會跑的 **Rust 後端 + PostgreSQL 資料庫**(登入、預約、上傳都要它),Pages 跑不起來。所以我們用:

| 元件 | 平台 | 免費額度 |
|------|------|----------|
| App(Rust 後端 + 前端靜態檔) | **Koyeb** | 1 個 web service,免綁信用卡 |
| 資料庫(PostgreSQL) | **Neon** | 3 GiB serverless,不會 30 天過期 |

兩者都吃 Docker,repo 裡的 `Dockerfile` 一份到處都能用(要換 Render / Railway / Fly 也行)。

---

## Step 0 — 推上 GitHub

Koyeb 會從你的 GitHub repo 直接 build。先用**你自己的帳號**建一個 repo(例如 `yoga_booking`),然後:

```sh
git remote add origin https://github.com/<你的帳號>/yoga_booking.git
git branch -M main
git push -u origin main
```

> `.gitignore` 已排除 `target/`、`node_modules/`、`frontend/dist/`,不會把肥檔案推上去。

---

## Step 1 — 建資料庫(Neon)

1. 到 <https://neon.tech> 用 GitHub 登入,建一個 project(region 挑離你近的,例如 Singapore)。
2. 建好後複製 **Connection string**,長這樣:

   ```
   postgresql://<user>:<password>@ep-xxxx-xxxx.<region>.aws.neon.tech/<dbname>?sslmode=require
   ```

   把它整條記下來,等一下當 `DATABASE_URL`。`?sslmode=require` 一定要留著(Neon 只收 TLS 連線,app 已內建 rustls 支援)。

---

## Step 2 — 部署 App(Koyeb)

1. 到 <https://koyeb.com> 用 GitHub 登入 → **Create Web Service** → 選 **GitHub** → 選你剛推的 repo。
2. Koyeb 會自動偵測到 `Dockerfile`,**Builder 選 Dockerfile**(不要選 Buildpack)。
3. **Instance** 選免費的 `Free`(Eco / Nano)。
4. **Port**:設成 `5150`(容器就聽這個 port)。
5. **Health check**(可留預設,或設得更準):HTTP path `/_ping`。
6. **Environment variables** 設下面這幾個:

   | 變數 | 值 | 說明 |
   |------|-----|------|
   | `DATABASE_URL` | Step 1 的 Neon 連線字串 | 必填 |
   | `JWT_SECRET`   | 隨機長字串,用 `openssl rand -hex 32` 產 | 必填,簽登入 token |
   | `HOST`         | `https://<你的app>.koyeb.app` | 必填,信件連結會用到 |

   > `LOCO_ENV=production`、`PORT=5150` 已寫在 `Dockerfile` 裡,不用重設。
   > 信件預設是 **stub**(不真的寄,記在記憶體),所以 demo 不需要任何 SMTP 設定,註冊也能正常跑。要寄真信才另外設 `MAILER_STUB=false` + `MAILER_HOST` / `MAILER_USER` / `MAILER_PASSWORD`。

7. 按 **Deploy**。第一次開機 `auto_migrate` 會自動在 Neon 建好所有資料表(不用手動 migrate)。

Build 第一次比較久(Rust release build 要幾分鐘),之後有 cache 會快很多。

---

## Step 3 — 建第一間工作室 + 管理員

資料表建好後是空的,還沒有任何工作室或帳號。因為 Neon 從任何地方都連得到,**在你自己電腦上**對著 Neon 跑兩個 task 就好(需要本機有 Rust 工具鏈,你開發時已經有了):

```sh
# 把 <neon-url> 換成 Step 1 的連線字串
export DATABASE_URL="<neon-url>"
export JWT_SECRET="whatever-nonempty"   # task 用不到,但 production config 要求有值
export HOST="https://<你的app>.koyeb.app"
export LOCO_ENV=production

# 1) 建一間工作室
cargo loco task organization:create name:"瑜安伽 Yuan Yoga" timezone:"Asia/Taipei"
#   → 會印出： id: 1  和  註冊連結 /register/<一串亂碼 public_id>

# 2) 建一個管理員(organization_id 用上面印出的 id)
cargo loco task user:create \
  email:you@example.com name:"Admin" password:"a-good-password" \
  organization_id:1 role:admin
```

- `organization:create` 會印出這間店的**註冊連結** `/register/<public_id>`(一串亂碼,不是流水號),把它給要加入的老師 / 學生,他們才能註冊進**這一間**。
- `role` 可以是 `member`(學生,預設)/ `staff`(老師,可開課)/ `admin`(管理員,可進後台開新工作室)。

> 也可以直接在本機跑這兩個 task 先把資料建好,再部署 Koyeb —— 先後順序都行,反正大家連的是同一個 Neon。

---

## Step 4 — 打開來用

到 `https://<你的app>.koyeb.app`:

- 首頁 → 用 Step 3 的管理員帳號登入。
- 管理員可進 **/admin** 後台,再開更多工作室、發註冊連結。
- 每間店的公開頁在 `/studio/{public_id}`,學生從 `/register/{public_id}` 註冊後就能預約。

---

## 幾個要知道的限制(免費方案)

- **上傳的圖片不會永久保存**:老師照片 / 工作室頁圖片存在容器本機磁碟(local storage),Koyeb 重新部署會清空 → 圖片會不見。demo 夠用;要永久保存得改接物件儲存(S3 之類),之後再說。
- **信件是 stub**:不會真的寄出。要真寄信照 Step 2 表格下方設 `MAILER_STUB=false` + SMTP 變數。
- **免費 instance 可能會冷啟動**:一陣子沒人用可能會被縮到 0,第一個請求會慢個幾秒喚醒。
- **只跑一個 instance**:`auto_migrate` 開著沒問題;如果之後要跑多個 instance,要改成用獨立步驟 migrate(把 `DB_AUTO_MIGRATE` 關掉),避免多台同時 migrate 打架。
