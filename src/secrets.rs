//! Secret values are execution inputs, never public Manifest fields.
use crate::proto::wafl::Manifest;
use connectrpc::ConnectError;
use std::collections::BTreeMap;

pub(crate) const COUNT: usize = 64;
pub(crate) const VALUE_BYTES: usize = 16 * 1024;
pub(crate) const TOTAL_BYTES: usize = 64 * 1024;

pub(crate) fn env_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name != "PORT"
        && name
            .bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || i > 0 && b.is_ascii_digit())
}

#[derive(Clone, Default)]
pub(crate) struct Values(pub BTreeMap<String, String>);
impl std::fmt::Debug for Values {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted secret environment]")
    }
}
impl Values {
    pub fn parse(
        values: impl IntoIterator<Item = (String, String)>,
        manifest: &Manifest,
    ) -> Result<Self, ConnectError> {
        let values: BTreeMap<_, _> = values.into_iter().collect();
        if values.len() > COUNT
            || values.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>() > TOTAL_BYTES
        {
            return Err(ConnectError::invalid_argument(
                "secret environment exceeds limits",
            ));
        }
        if values.iter().any(|(k, v)| {
            !env_name(k)
                || manifest.env.contains_key(k)
                || v.len() > VALUE_BYTES
                || v.contains('\0')
        }) {
            return Err(ConnectError::invalid_argument(
                "invalid secret environment or collision with manifest.env",
            ));
        }
        Ok(Self(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn values_preserve_bytes_validate_bounds_and_redact_debug() {
        let parse = |name: &str, value: &str, manifest: &Manifest| {
            Values::parse([(name.into(), value.into())], manifest)
        };
        let manifest = Manifest::default();
        let values = parse("TOKEN", "fake-sensitive-value\n", &manifest).unwrap();
        assert_eq!(values.0["TOKEN"], "fake-sensitive-value\n");
        assert!(!format!("{values:?}").contains("fake-sensitive"));
        assert!(parse("TOKEN", "", &manifest).is_ok());
        for name in ["", "PORT", "1TOKEN", "A=B", "BAD-NAME", "한글"] {
            assert!(parse(name, "test", &manifest).is_err());
        }
        assert!(parse("TOKEN", "nul\0value", &manifest).is_err());
        assert!(parse("TOKEN", &"a".repeat(VALUE_BYTES + 1), &manifest).is_err());
        let mut collision = manifest.clone();
        collision.env.insert("TOKEN".into(), "public".into());
        assert!(parse("TOKEN", "test", &collision).is_err());
        assert!(
            Values::parse((0..65).map(|n| (format!("V{n}"), String::new())), &manifest).is_err()
        );
        assert!(
            Values::parse(
                (0..5).map(|n| (format!("V{n}"), "a".repeat(VALUE_BYTES))),
                &manifest
            )
            .is_err()
        );
    }
}
