mod graphql;
mod errors;

use std::env;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use app::aws::ssm;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ssm::load_env().await;

    let port = env::var("PORT").unwrap_or("8000".to_string());
    let with_lambda: bool = env::var("WITH_LAMBDA")
        .unwrap_or("false".to_string())
        .parse()
        .expect("failed to parse WITH_LAMBDA");
    let with_playground: bool = env::var("WITH_PLAYGROUND")
        .unwrap_or("false".to_string())
        .parse()
        .expect("failed to parse WITH_PLAYGROUND");

    let handler = graphql::HttpHandler::new();

    let app_factory = move || {
        let mut app = App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allowed_methods(["GET", "POST"])
                    .max_age(3600)
                    .supports_credentials(),
            )
            .app_data(Data::new(handler.clone()))
            .service(
                web::resource("/graphql")
                    .guard(guard::Post())
                    .to(graphql_route),
            );
        if with_playground {
            app = app.service(
                web::resource("/playground")
                    .guard(guard::Get())
                    .to(playground_route),
            );
        }
        app
    };

    if with_lambda {
        lambda_web::run_actix_on_lambda(app_factory)
            .await
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
    } else {
        println!("listen as http server on port {}", port);
        HttpServer::new(app_factory)
            .bind(format!("127.0.0.1:{}", port))?
            .run()
            .await
    }
}

async fn graphql_route(
    handler: Data<graphql::HttpHandler>,
    http_req: HttpRequest,
    gql_req: GraphQLRequest,
) -> GraphQLResponse {
    handler.handle(http_req, gql_req).await
}

async fn playground_route() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
        )))
}