use crate::error::DbError;
use crate::executor::context::ExecutionContext;
use crate::executor::metadata::{type_max_length, type_name};
use crate::executor::result::QueryResult;
use crate::executor::script::ScriptExecutor;
use crate::types::{DataType, Value};

pub(crate) fn execute_sp_describe_first_result_set(
    exec: &mut ScriptExecutor<'_>,
    args: &[String],
    ctx: &mut ExecutionContext<'_>,
) -> Result<QueryResult, DbError> {
    if args.is_empty() {
        return Err(DbError::Execution(
            "sp_describe_first_result_set requires 1 argument: @tsql".into(),
        ));
    }

    let tsql = &args[0];
    if tsql.trim().is_empty() {
        return Err(DbError::Execution(
            "sp_describe_first_result_set: @tsql cannot be empty".into(),
        ));
    }

    // Parse and execute the query to get column metadata from the first result set.
    let batch = crate::parser::parse_batch(tsql)?;
    let outcome = exec.execute_batch(&batch, ctx)?;

    match outcome {
        crate::error::StmtOutcome::Ok(Some(result)) => {
            let mut rows = Vec::new();
            for (i, _col_name) in result.columns.iter().enumerate() {
                let dt = result.column_types.get(i).cloned().unwrap_or(DataType::Int);
                let is_nullable = result.column_nullabilities.get(i).copied().unwrap_or(true);
                let system_type_id = crate::executor::metadata::system_type_id(&dt);
                let system_type_name = type_name(&dt);
                let max_length = type_max_length(&dt) as i16;
                let (precision, scale) = match &dt {
                    DataType::Decimal { precision, scale } => {
                        (*precision as i8, *scale as i8)
                    }
                    _ => (0i8, 0i8),
                };
                let collation_name = match &dt {
                    DataType::NVarChar { .. }
                    | DataType::NChar { .. }
                    | DataType::VarChar { .. }
                    | DataType::Char { .. } => Some("SQL_Latin1_General_CP1_CI_AS".to_string()),
                    _ => None,
                };

                rows.push(vec![
                    Value::Bit(is_nullable),
                    Value::TinyInt(system_type_id as u8),
                    Value::NVarChar(system_type_name),
                    Value::SmallInt(max_length),
                    Value::TinyInt(precision as u8),
                    Value::TinyInt(scale as u8),
                    match collation_name {
                        Some(cn) => Value::NVarChar(cn),
                        None => Value::Null,
                    },
                ]);
            }

            Ok(QueryResult {
                columns: vec![
                    "IsNullable".into(),
                    "SystemTypeId".into(),
                    "SystemTypeName".into(),
                    "MaxLength".into(),
                    "Precision".into(),
                    "Scale".into(),
                    "CollationName".into(),
                ],
                column_types: vec![
                    DataType::Bit,
                    DataType::TinyInt,
                    DataType::NVarChar { max_len: 128 },
                    DataType::SmallInt,
                    DataType::TinyInt,
                    DataType::TinyInt,
                    DataType::NVarChar { max_len: 128 },
                ],
                column_nullabilities: vec![false, false, false, false, false, false, true],
                rows,
                ..Default::default()
            })
        }
        crate::error::StmtOutcome::Ok(None) => {
            // DDL or non-SELECT — return empty result set
            Ok(QueryResult {
                columns: vec![
                    "IsNullable".into(),
                    "SystemTypeId".into(),
                    "SystemTypeName".into(),
                    "MaxLength".into(),
                    "Precision".into(),
                    "Scale".into(),
                    "CollationName".into(),
                ],
                column_types: vec![
                    DataType::Bit,
                    DataType::TinyInt,
                    DataType::NVarChar { max_len: 128 },
                    DataType::SmallInt,
                    DataType::TinyInt,
                    DataType::TinyInt,
                    DataType::NVarChar { max_len: 128 },
                ],
                column_nullabilities: vec![false, false, false, false, false, false, true],
                rows: Vec::new(),
                ..Default::default()
            })
        }
        _ => Err(DbError::Execution(
            "sp_describe_first_result_set: query must produce a result set".into(),
        )),
    }
}

pub(crate) fn execute_sp_describe_undeclared_parameters(
    _exec: &mut ScriptExecutor<'_>,
    args: &[String],
    _ctx: &mut ExecutionContext<'_>,
) -> Result<QueryResult, DbError> {
    if args.is_empty() {
        return Err(DbError::Execution(
            "sp_describe_undeclared_parameters requires 1 argument: @tsql".into(),
        ));
    }

    // Return empty result set — iridium does not use undeclared parameters
    // in the same way as SQL Server's parameterized execution model.
    Ok(QueryResult {
        columns: vec![
            "ParameterName".into(),
            "SuggestedSystemType".into(),
            "SuggestedMaxLength".into(),
            "SuggestedPrecision".into(),
            "SuggestedScale".into(),
            "ParameterOrderNumber".into(),
            "IsOutput".into(),
            "HasDefault".into(),
            "DefaultInfo".into(),
            "ParameterDeclaredDataTypeText".into(),
        ],
        column_types: vec![
            DataType::NVarChar { max_len: 128 },
            DataType::NVarChar { max_len: 128 },
            DataType::SmallInt,
            DataType::TinyInt,
            DataType::TinyInt,
            DataType::Int,
            DataType::Bit,
            DataType::Bit,
            DataType::NVarChar { max_len: 256 },
            DataType::NVarChar { max_len: 256 },
        ],
        column_nullabilities: vec![
            false, false, false, false, false, false, false, false, true, true,
        ],
        rows: Vec::new(),
        ..Default::default()
    })
}
