
pub mod from_config;
pub mod provider;

#[cfg(test)]
mod test {
    use super::{from_config::FromConfig, provider::Provider};

    // config and provide
    #[derive(Debug, Clone, Copy)]
    struct Database;

    struct Config {
        db: Database,
    }

    impl<'r> Provider<'r, Database> for Config {
        fn provide(&'r self) -> Database {
            self.db
        }
    }

    impl<'r> Provider<'r, &'r Database> for Config {
        fn provide(&'r self) -> &'r Database {
            &self.db
        }
    }

    struct DbConfig(Database);

    impl<'r, C> FromConfig<'r, C> for DbConfig
    where
        C: Provider<'r, Database>,
    {
        fn from_config(config: &'r C) -> Self {
            Self(config.provide())
        }
    }
    #[test]
    fn test_nest_config() {
        let config = Config { db: Database };

        let _data = <(DbConfig, DbConfig, DbConfig) as FromConfig<Config>>::from_config(&config);
    }
}
