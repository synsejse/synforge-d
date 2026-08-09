macro_rules! impl_text_enum {
    ($name:ident {
        $($variant:ident => [$primary:literal $(, $alias:literal)*]),+ $(,)?
    }) => {
        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $primary,)+
                }
            }

            fn from_text(value: &str) -> Option<Self> {
                match value {
                    $($primary $(| $alias)* => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::from_text(value).ok_or(())
            }
        }

        #[cfg(feature = "diesel")]
        impl<DB> diesel::serialize::ToSql<diesel::sql_types::Text, DB> for $name
        where
            DB: diesel::backend::Backend,
            str: diesel::serialize::ToSql<diesel::sql_types::Text, DB>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, DB>,
            ) -> diesel::serialize::Result {
                self.as_str().to_sql(out)
            }
        }

        #[cfg(feature = "diesel")]
        impl<DB> diesel::deserialize::FromSql<diesel::sql_types::Text, DB> for $name
        where
            DB: diesel::backend::Backend,
            String: diesel::deserialize::FromSql<diesel::sql_types::Text, DB>,
        {
            fn from_sql(value: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
                let value = String::from_sql(value)?;
                Self::from_text(&value).ok_or_else(|| {
                    format!("invalid {} value: {}", stringify!($name), value).into()
                })
            }
        }
    };
}

pub(crate) use impl_text_enum;
