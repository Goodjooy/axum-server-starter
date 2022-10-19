pub mod provider;

#[cfg(test)]
mod test {
    use super::provider::Provider;

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

    #[test]
    fn test_nest_config() {
        let config = Config { db: Database };

        let _data = <Config as Provider<(Database, Database, Database)>>::provide(&config);
    }
}
