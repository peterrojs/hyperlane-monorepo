use crate::model::matching_list::MatchingList;
use convert_case::Case;
use eyre::{eyre, Context};
use hyperlane_base::settings::parser::{recase_json_value, ValueParser};
use hyperlane_core::config::{
    ConfigErrResultExt, ConfigParsingError, ConfigPath, ConfigResult, ConfigResultExt,
};
use serde_json::Value;

pub mod matching_list;
pub mod send_args;

fn parse_json_array(p: ValueParser) -> Option<(ConfigPath, Value)> {
    let mut err = ConfigParsingError::default();

    match p {
        ValueParser {
            val: Value::String(array_str),
            cwp,
        } => serde_json::from_str::<Value>(array_str)
            .context("Expected JSON string")
            .take_err(&mut err, || cwp.clone())
            .map(|v| (cwp, recase_json_value(v, Case::Flat))),
        ValueParser {
            val: value @ Value::Array(_),
            cwp,
        } => Some((cwp, value.clone())),
        _ => Err(eyre!("Expected JSON array or stringified JSON"))
            .take_err(&mut err, || p.cwp.clone()),
    }
}

fn parse_matching_list(p: ValueParser) -> ConfigResult<MatchingList> {
    let mut err = ConfigParsingError::default();

    let raw_list = parse_json_array(p.clone()).map(|(_, v)| v);
    let Some(raw_list) = raw_list else {
        return err.into_result(MatchingList::default());
    };
    let p = ValueParser::new(p.cwp.clone(), &raw_list);
    let ml = p
        .parse_value::<MatchingList>("Expected matching list")
        .take_config_err(&mut err)
        .unwrap_or_default();

    err.into_result(ml)
}
