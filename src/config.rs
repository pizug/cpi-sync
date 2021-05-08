use serde::{Deserialize, Serialize};

fn default_package_rule_operation() -> OperationEnum {
    OperationEnum::Include
}
fn default_extract_zip() -> ZipExtraction {
    ZipExtraction::Enabled
}

fn default_prop_comment_removal() -> PropCommentRemoval {
    PropCommentRemoval::Disabled
}

fn default_packages_local_dir() -> String {
    "".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OperationEnum {
    #[serde(rename = "include")]
    Include,
    #[serde(rename = "exclude")]
    Exclude,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageSingle {
    pub id: String,
    #[serde(default = "default_package_rule_operation")]
    pub operation: OperationEnum,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageRegex {
    #[serde(default = "default_package_rule_operation")]
    pub operation: OperationEnum,
    pub pattern: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum PackageRuleEnum {
    #[serde(rename = "regex")]
    Regex(PackageRegex),
    #[serde(rename = "single")]
    Single(PackageSingle),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ZipExtraction {
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "enabled")]
    Enabled,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PropCommentRemoval {
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "enabled")]
    Enabled,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packages {
    #[serde(default = "default_extract_zip")]
    pub zip_extraction: ZipExtraction,
    #[serde(default = "default_prop_comment_removal")]
    pub prop_comment_removal: PropCommentRemoval,
    #[serde(default = "default_packages_local_dir")]
    pub local_dir: String,
    pub filter_rules: Vec<PackageRuleEnum>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CredentialSUser {
    pub username: String,
    pub password_environment_variable: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CredentialOauthClientCredentials {
    pub client_id: String,
    pub token_endpoint_url: String,
    pub client_secret_environment_variable: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CredentialInside {
    #[serde(rename = "oauth_client_credentials")]
    OauthClientCredentials(CredentialOauthClientCredentials),
    #[serde(rename = "s_user")]
    SUser(CredentialSUser),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tenant {
    pub management_host: String,
    pub credential: CredentialInside,
    // credential: CredentialInside,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub cpisync: String,
    pub tenant: Tenant,
    pub packages: Packages,
}
