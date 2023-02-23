use std::collections::HashMap;

use tokio_postgres::Config;

const DEFAULT_PG_SOCKET_FILE: &str = "/var/run/postgresql/s.PGSQL";

#[derive(Debug, Default)]
pub struct ConfigBuilder {
    settings: HashMap<String, String>,
}

impl ConfigBuilder {
    pub fn add_setting(&mut self, name: &str, setting: &str) -> &mut Self {
        self.settings.insert(name.to_string(), setting.to_string());

        self
    }

    pub fn build(self, auth_token: AuthenticationToken) -> Config {
        let mut config = Config::new();
        config.user(&auth_token.username);

        for host in self
            .settings
            .get("host")
            .map(|v| v.as_str())
            .unwrap_or(DEFAULT_PG_SOCKET_FILE)
            .split(",")
        {
            config.host(host.trim());
        }

        if let Some(password) = auth_token.password {
            config.password(&password);
        }

        if let Some(application_name) = self.settings.get("application_name") {
            config.application_name(application_name);
        }

        config
    }
}

#[derive(Debug)]
pub struct AuthenticationToken {
    username: String,
    password: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_auth_token(password: Option<String>) -> AuthenticationToken {
        AuthenticationToken {
            username: "pikachu".to_string(),
            password,
        }
    }

    #[test]
    fn default_config() {
        let config = ConfigBuilder::default().build(get_auth_token(None));

        assert_eq!(vec![Host::Unix(DEFAULT_PG_SOCKET_FILE)], config.get_hosts());
    }
}
