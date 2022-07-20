mod query;
mod mutation;

use std::{net};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use actix_web::HttpRequest;
use async_graphql::{EmptySubscription, Request, Response, ServerError, ServerResult, ValidationResult};
use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextPrepareRequest, NextRequest, NextResolve, NextValidation, ResolveInfo};
use async_graphql::parser::types::ExecutableDocument;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use async_graphql_value::{ConstValue, Variables};
use crate::errors::{FieldErrorCode};
use crate::graphql::mutation::MutationRoot;
use crate::graphql::query::QueryRoot;
use async_trait::async_trait;
use serde::Serialize;

pub type Schema = async_graphql::Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(Clone)]
pub struct HttpHandler {
    pub schema: Schema,
}

impl HttpHandler {
    pub fn new() -> Self {
        let schema = Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
            .extension(ExtFactory {})
            .finish();

        HttpHandler {
            schema,
        }
    }

    pub async fn handle(&self, http_req: HttpRequest, gql_req: GraphQLRequest) -> GraphQLResponse {
        let mut gql_req = gql_req.into_inner();
        if let Some(v) = http_req.connection_info().realip_remote_addr() {
            if let Ok(v) = net::IpAddr::from_str(v) {
                gql_req = gql_req.data(RemoteIpAddr(v))
            }
        }
        self.schema.execute(gql_req).await.into()
    }
}

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct AccessLogEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_ip: Option<net::IpAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct RemoteIpAddr(pub net::IpAddr);

pub struct ExtFactory {

}

impl ExtensionFactory for ExtFactory {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(Ext {
            access_log_entry: RwLock::new(AccessLogEntry::default()),
        })
    }
}

struct Ext {
    access_log_entry: RwLock<AccessLogEntry>,
}

#[async_trait]
impl Extension for Ext {
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        let res = next.run(ctx).await;

        let log_entry = self.access_log_entry.read().unwrap().clone();
        match log_entry.query {
            Some(v) if v.contains("__schema") => {

            }
            _ => {
                if let Ok(v) = serde_json::to_string(&log_entry) {
                    println!("{}", v);
                }
            }
        };

        res
    }

    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        {
            let mut log = self.access_log_entry.write().unwrap();
            log.remote_ip = ctx.data::<RemoteIpAddr>().ok().map(|v| v.0.clone());
            log.query = Some(request.query.clone());
            log.variables = match serde_json::to_value(request.variables.clone()).unwrap() {
                serde_json::Value::Object(v) if v.is_empty() => None,
                v => Some(v),
            };
            log.op_name = request.operation_name.clone()
        }

        let res = next.run(ctx, request).await;
        if let Err(ref err) = res {
            self.access_log_entry.write().unwrap().error =
                Some(format!("failed to prepare request: {}", err));
        }

        res
    }

    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let res = next.run(ctx, query, variables).await;

        if let Err(ref err) = res {
            self.access_log_entry.write().unwrap().error =
                Some(format!("failed to parse query: {}", err));
        }

        res
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let res = next.run(ctx).await;

        if let Err(ref err) = res {
            self.access_log_entry.write().unwrap().error = Some(format!(
                "failed to validation: {}",
                err.into_iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        res
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let res = next.run(ctx, operation_name).await;

        if !res.errors.is_empty() {
            let access_info = self.access_log_entry.read().unwrap().clone();

            let _info: BTreeMap<String, serde_json::Value> = [
                ("ip", access_info.remote_ip.clone().map(|v| serde_json::Value::String(v.to_string()))),
                ("op_name", access_info.op_name.clone().map(|v| serde_json::Value::String(v))),
                (
                    "query",
                    access_info
                        .query
                        .clone()
                        .map(|v| serde_json::Value::String(v)),
                ),
                ("variables", access_info.variables.clone()),
            ]
                .into_iter()
                .flat_map(|(k, v)| v.map(|v| (k.to_string(), v)))
                .collect();

            for err in &res.errors {
                let msg = serde_json::to_string(err).unwrap();

                match &err.extensions {
                    Some(ex)
                    if ex
                        .get("code")
                        .eq(&Some(&FieldErrorCode::Internal.into())) =>
                        {
                            // エラーログ
                            println!("{}", msg);
                        }
                    _ => {
                        println!("{}", msg);
                    },
                }
            }
        }

        res
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<ConstValue>> {
        next.run(ctx, info).await
    }
}