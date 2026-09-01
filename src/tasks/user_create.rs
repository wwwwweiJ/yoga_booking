use loco_rs::prelude::*;

use crate::{
    mailers::auth::AuthMailer,
    models::{_entities::users, users::RegisterParams},
};

pub struct UserCreate;
#[async_trait]
impl Task for UserCreate {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "user:create".to_string(),
            detail: "Create a new user with email, name, password, and organization_id (role optional: member|staff|admin, default member). Sends welcome email and sets up email verification.\nUsage:\ncargo run task user:create email:user@example.com name:\"John Doe\" password:\"securepassword\" organization_id:1 role:staff".to_string(),
        }
    }
    async fn run(&self, app_context: &AppContext, vars: &task::Vars) -> Result<()> {
        let email = vars
            .cli_arg("email")
            .map_err(|_| Error::string("email is mandatory"))?;
        let name = vars
            .cli_arg("name")
            .map_err(|_| Error::string("name is mandatory"))?;
        let password = vars
            .cli_arg("password")
            .map_err(|_| Error::string("password is mandatory"))?;
        let organization_id = vars
            .cli_arg("organization_id")
            .map_err(|_| Error::string("organization_id is mandatory"))?
            .parse::<i64>()
            .map_err(|_| Error::string("organization_id must be an integer"))?;
        // Optional; defaults to a plain member (student).
        let role = vars.cli_arg("role").unwrap_or("member").to_owned();

        let register_params = RegisterParams {
            email: email.to_owned(),
            password: password.to_owned(),
            name: name.to_owned(),
            organization_id,
        };

        // Create user with password using the same logic as register controller
        let res = users::Model::create_with_password(&app_context.db, &register_params).await;

        let user = match res {
            Ok(user) => {
                tracing::info!(
                    message = "User created successfully",
                    user_email = &register_params.email,
                    user_pid = user.pid.to_string(),
                    "user created via task"
                );
                user
            }
            Err(err) => {
                tracing::error!(
                    message = err.to_string(),
                    user_email = &register_params.email,
                    "could not create user via task"
                );
                return Err(Error::string(&format!("Failed to create user. err: {err}")));
            }
        };

        // Promote to the requested tier (create_with_password always makes a
        // member; operators use this task to mint staff / admin accounts).
        let user = if role == "member" {
            user
        } else {
            let mut active = user.into_active_model();
            active.role = Set(role.clone());
            active
                .update(&app_context.db)
                .await
                .map_err(|e| Error::string(&format!("Failed to set role. err: {e}")))?
        };

        // Set email verification sent (same as register controller)
        let user = user
            .into_active_model()
            .set_email_verification_sent(&app_context.db)
            .await
            .map_err(|err| {
                tracing::error!(
                    message = err.to_string(),
                    user_email = &register_params.email,
                    "could not set email verification"
                );
                Error::string("Failed to set email verification")
            })?;

        // Send welcome email (same as register controller)
        AuthMailer::send_welcome(app_context, &user)
            .await
            .map_err(|err| {
                tracing::error!(
                    message = err.to_string(),
                    user_email = &register_params.email,
                    "could not send welcome email"
                );
                Error::string("Failed to send welcome email")
            })?;

        tracing::info!(
            message = "User creation task completed successfully",
            user_email = &register_params.email,
            user_pid = user.pid.to_string(),
            "user creation task finished"
        );

        println!("✅ User created successfully!");
        println!("   Email: {}", user.email);
        println!("   Name: {}", user.name);
        println!("   Role: {}", user.role);
        println!("   PID: {}", user.pid);

        Ok(())
    }
}
