use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,       // SSH config: Host alias
    pub host: String,       // SSH config: HostName
    pub port: u16,          // SSH config: Port
    pub username: String,   // SSH config: User
    pub auth_type: AuthType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_file: Option<String>, // SSH config: IdentityFile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Password,
    Key,
}

impl Server {
    pub fn new(name: String, host: String, username: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            host,
            port: 22,
            username,
            auth_type: AuthType::Password,
            identity_file: None,
            password: None,
            private_key: None,
            passphrase: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn ssh_config_block(&self) -> String {
        let mut buf = format!("Host {}\n    HostName {}\n", self.name, self.host);
        if self.port != 22 {
            buf.push_str(&format!("    Port {}\n", self.port));
        }
        buf.push_str(&format!("    User {}\n", self.username));
        if let Some(ref id_file) = self.identity_file {
            if !id_file.is_empty() {
                buf.push_str(&format!("    IdentityFile {}\n", id_file));
            }
        }
        buf
    }
}

impl std::fmt::Display for AuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthType::Password => write!(f, "password"),
            AuthType::Key => write!(f, "key"),
        }
    }
}
