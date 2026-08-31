use loco_rs::prelude::*;

use crate::models::_entities::organizations;

/// Studios (organizations) are created by an operator, not self-service — users
/// register *into* an existing studio. This task is that operator entry point.
pub struct OrganizationCreate;

#[async_trait]
impl Task for OrganizationCreate {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "organization:create".to_string(),
            detail: "Create a studio (organization).\nUsage:\ncargo run task organization:create name:\"Sunrise Yoga\" timezone:\"Asia/Taipei\"".to_string(),
        }
    }

    async fn run(&self, app_context: &AppContext, vars: &task::Vars) -> Result<()> {
        let name = vars
            .cli_arg("name")
            .map_err(|_| Error::string("name is mandatory"))?;
        let timezone = vars
            .cli_arg("timezone")
            .map_err(|_| Error::string("timezone is mandatory"))?;

        let org = organizations::ActiveModel {
            name: ActiveValue::set(name.to_owned()),
            timezone: ActiveValue::set(timezone.to_owned()),
            ..Default::default()
        }
        .insert(&app_context.db)
        .await
        .map_err(|err| Error::string(&format!("Failed to create organization: {err}")))?;

        println!("✅ Organization created!");
        println!("   id: {}", org.id);
        println!("   name: {}", org.name);
        println!("   timezone: {}", org.timezone);
        println!();
        println!("Share this register link with its members:");
        println!("   /register/{}", org.public_id);

        Ok(())
    }
}
