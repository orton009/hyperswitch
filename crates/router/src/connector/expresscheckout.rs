mod transformers;

use std::fmt::Debug;
use error_stack::{ResultExt, IntoReport};

use crate::{
    configs::settings,
    utils::{self, BytesExt},
    core::{
        errors::{self, CustomResult},
        payments::{self, access_token},
    },
    headers, logger, services::{self, ConnectorIntegration},
    types::{
        self,
        api::{self, ConnectorCommon, ConnectorCommonExt},
        ErrorResponse, Response,
    }
};


use transformers as expresscheckout;

#[derive(Debug, Clone)]
pub struct Expresscheckout;

impl<Flow, Request, Response> ConnectorCommonExt<Flow, Request, Response> for Expresscheckout
where
    Self: ConnectorIntegration<Flow, Request, Response>,{
    fn build_headers(
        &self,
        req: &types::RouterData<Flow, Request, Response>,
        _connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, String)>, errors::ConnectorError> {
        let mut headers = vec![
            ( headers::CONTENT_TYPE.to_string(),
              self.get_content_type().to_string()
            ),
            ("x-merchantid".to_string(), req.merchant_id.clone())
        ];
        let auth = expresscheckout::ExpresscheckoutAuthType::try_from(&req.connector_auth_type)?;
        let auth_header = (
            headers::AUTHORIZATION.to_string(),
            format!("Basic {}", auth.api_key),
        );
        headers.push(auth_header);
        Ok(headers)
    }
}

impl ConnectorCommon for Expresscheckout {
    fn id(&self) -> &'static str {
        "expresscheckout"
    }

    fn common_get_content_type(&self) -> &'static str {
        "application/x-www-form-urlencoded"
    }

    fn base_url<'a>(&self, connectors: &'a settings::Connectors) -> &'a str {
        connectors.expresscheckout.base_url.as_ref()
    }

    // fn get_auth_header(&self, auth_type:&types::ConnectorAuthType)-> CustomResult<Vec<(String,String)>,errors::ConnectorError> {
    //     let auth: expresscheckout::ExpresscheckoutAuthType = auth_type
    //         .try_into()
    //         .change_context(errors::ConnectorError::FailedToObtainAuthType)?;
    //     Ok(vec![(headers::AUTHORIZATION.to_string(), auth.api_key)])
    // }
}

impl api::Payment for Expresscheckout {}

impl api::PreVerify for Expresscheckout {}
impl
    ConnectorIntegration<
        api::Verify,
        types::VerifyRequestData,
        types::PaymentsResponseData,
    > for Expresscheckout
{
}

impl api::PaymentVoid for Expresscheckout {}

impl
    ConnectorIntegration<
        api::Void,
        types::PaymentsCancelData,
        types::PaymentsResponseData,
    > for Expresscheckout
{}

impl api::ConnectorAccessToken for Expresscheckout {}

impl ConnectorIntegration<api::AccessTokenAuth, types::AccessTokenRequestData, types::AccessToken>
    for Expresscheckout
{
}

impl api::PaymentSync for Expresscheckout {}
impl
    ConnectorIntegration<api::PSync, types::PaymentsSyncData, types::PaymentsResponseData>
    for Expresscheckout
{
    fn get_headers(
        &self,
        req: &types::PaymentsSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, String)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        req: &types::PaymentsSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        let url = format!("{}{}{}",self.base_url(connectors), "orders/", req.payment_id.clone());
        println!("building sync {:#?}", url);
        Ok(url)
    }

    fn build_request(
        &self,
        req: &types::PaymentsSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        let body = services::RequestBuilder::new()
        .method(services::Method::Get)
        .url(&types::PaymentsSyncType::get_url(self, req, connectors)?)
        .headers(types::PaymentsSyncType::get_headers(self, req, connectors)?)
        .build();
        println!("sync body {:#?}", body);
        Ok(Some(body))
    }

    fn get_error_response(
        &self,
        res: Response,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res)
    }

    fn handle_response(
        &self,
        data: &types::PaymentsSyncRouterData,
        res: Response,
    ) -> CustomResult<types::PaymentsSyncRouterData, errors::ConnectorError> {
        println!("sync response {:#?}", res.response);
        logger::debug!(payment_sync_response=?res);
        let response: expresscheckout:: ExpresscheckoutPaymentsResponse = res
            .response
            .parse_struct("expresscheckout PaymentsResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }
}


impl api::PaymentCapture for Expresscheckout {}
impl
    ConnectorIntegration<
        api::Capture,
        types::PaymentsCaptureData,
        types::PaymentsResponseData,
    > for Expresscheckout
{
    fn get_headers(
        &self,
        req: &types::PaymentsCaptureRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, String)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        _req: &types::PaymentsCaptureRouterData,
        _connectors: &settings::Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        todo!()
    }

    fn get_request_body(
        &self,
        _req: &types::PaymentsCaptureRouterData,
    ) -> CustomResult<Option<String>, errors::ConnectorError> {
        todo!()
    }

    fn build_request(
        &self,
        req: &types::PaymentsCaptureRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Post)
                .url(&types::PaymentsCaptureType::get_url(self, req, connectors)?)
                .headers(types::PaymentsCaptureType::get_headers(
                    self, req, connectors,
                )?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::PaymentsCaptureRouterData,
        res: Response,
    ) -> CustomResult<types::PaymentsCaptureRouterData, errors::ConnectorError> {
        let response: expresscheckout::ExpresscheckoutPaymentsResponse = res
            .response
            .parse_struct("Expresscheckout PaymentsResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        logger::debug!(expresscheckoutpayments_create_response=?response);
        types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        }
        .try_into()
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response(
        &self,
        res: Response,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res)
    }
}

impl api::PaymentSession for Expresscheckout {}

impl
    ConnectorIntegration<
        api::Session,
        types::PaymentsSessionData,
        types::PaymentsResponseData,
    > for Expresscheckout
{
    //TODO: implement sessions flow
}

impl api::PaymentAuthorize for Expresscheckout {}

impl
    ConnectorIntegration<
        api::Authorize,
        types::PaymentsAuthorizeData,
        types::PaymentsResponseData,
    > for Expresscheckout {
    fn get_headers(&self, req: &types::PaymentsAuthorizeRouterData, connectors: &settings::Connectors,) -> CustomResult<Vec<(String, String)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(&self, _req: &types::PaymentsAuthorizeRouterData, connectors: &settings::Connectors,) -> CustomResult<String,errors::ConnectorError> {
        println!("[Api url] {:#?}", self.base_url(connectors));
        Ok(format!(
            "{}{}",
            self.base_url(connectors),
            "txns/"
        ))
    }

    fn get_request_body(&self, req: &types::PaymentsAuthorizeRouterData) -> CustomResult<Option<String>,errors::ConnectorError> {
        println!("[req notEncoded] {:#?}", req);
        let expresscheckout_req =
            utils::Encode::<expresscheckout::ExpresscheckoutPaymentsRequest>::convert_and_url_encode(req).change_context(errors::ConnectorError::RequestEncodingFailed)?;
        println!("[req encoded] {:#?}", expresscheckout_req);
        Ok(Some(expresscheckout_req))
    }

    fn build_request(
        &self,
        req: &types::PaymentsAuthorizeRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Post)
                .url(&types::PaymentsAuthorizeType::get_url(
                    self, req, connectors,
                )?)
                .headers(types::PaymentsAuthorizeType::get_headers(
                    self, req, connectors,
                )?)
                .body(types::PaymentsAuthorizeType::get_request_body(self, req)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::PaymentsAuthorizeRouterData,
        res: Response,
    ) -> CustomResult<types::PaymentsAuthorizeRouterData,errors::ConnectorError> {
        let response: expresscheckout::ExpresscheckoutPaymentsResponse = res.response.parse_struct("PaymentIntentResponse").change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        logger::debug!(expresscheckoutpayments_create_response=?response);
        types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        }
        .try_into()
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response(&self, res: Response) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res)
    }
}

impl api::Refund for Expresscheckout {}
impl api::RefundExecute for Expresscheckout {}
impl api::RefundSync for Expresscheckout {}

impl
    ConnectorIntegration<
        api::Execute,
        types::RefundsData,
        types::RefundsResponseData,
    > for Expresscheckout {
    fn get_headers(&self, req: &types::RefundsRouterData<api::Execute>, connectors: &settings::Connectors,) -> CustomResult<Vec<(String,String)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(&self, _req: &types::RefundsRouterData<api::Execute>, _connectors: &settings::Connectors,) -> CustomResult<String,errors::ConnectorError> {
        todo!()
    }

    fn get_request_body(&self, req: &types::RefundsRouterData<api::Execute>) -> CustomResult<Option<String>,errors::ConnectorError> {
        let expresscheckout_req = utils::Encode::<expresscheckout::ExpresscheckoutRefundRequest>::convert_and_url_encode(req).change_context(errors::ConnectorError::RequestEncodingFailed)?;
        Ok(Some(expresscheckout_req))
    }

    fn build_request(&self, req: &types::RefundsRouterData<api::Execute>, connectors: &settings::Connectors,) -> CustomResult<Option<services::Request>,errors::ConnectorError> {
        let request = services::RequestBuilder::new()
            .method(services::Method::Post)
            .url(&types::RefundExecuteType::get_url(self, req, connectors)?)
            .headers(types::RefundExecuteType::get_headers(self, req, connectors)?)
            .body(types::RefundExecuteType::get_request_body(self, req)?)
            .build();
        Ok(Some(request))
    }

    fn handle_response(
        &self,
        data: &types::RefundsRouterData<api::Execute>,
        res: Response,
    ) -> CustomResult<types::RefundsRouterData<api::Execute>,errors::ConnectorError> {
        logger::debug!(target: "router::connector::expresscheckout", response=?res);
        let response: expresscheckout::RefundResponse = res.response.parse_struct("expresscheckout RefundResponse").change_context(errors::ConnectorError::RequestEncodingFailed)?;
        types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        }
        .try_into()
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response(&self, res: Response) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res)
    }
}

impl
    ConnectorIntegration<api::RSync, types::RefundsData, types::RefundsResponseData> for Expresscheckout {
    fn get_headers(&self, req: &types::RefundSyncRouterData,connectors: &settings::Connectors,) -> CustomResult<Vec<(String, String)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(&self, _req: &types::RefundSyncRouterData,_connectors: &settings::Connectors,) -> CustomResult<String,errors::ConnectorError> {
        todo!()
    }

    fn build_request(
        &self,
        req: &types::RefundSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Get)
                .url(&types::RefundSyncType::get_url(self, req, connectors)?)
                .headers(types::RefundSyncType::get_headers(self, req, connectors)?)
                .body(types::RefundSyncType::get_request_body(self, req)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::RefundSyncRouterData,
        res: Response,
    ) -> CustomResult<types::RefundSyncRouterData,errors::ConnectorError,> {
        logger::debug!(target: "router::connector::expresscheckout", response=?res);
        let response: expresscheckout::RefundResponse = res.response.parse_struct("expresscheckout RefundResponse").change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        }
        .try_into()
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response(&self, res: Response) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res)
    }
}

#[async_trait::async_trait]
impl api::IncomingWebhook for Expresscheckout {
    fn get_webhook_object_reference_id(
        &self,
        _body: &[u8],
    ) -> CustomResult<String, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }

    fn get_webhook_event_type(
        &self,
        _body: &[u8],
    ) -> CustomResult<api::IncomingWebhookEvent, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }

    fn get_webhook_resource_object(
        &self,
        _body: &[u8],
    ) -> CustomResult<serde_json::Value, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }
}

impl services::ConnectorRedirectResponse for Expresscheckout {
    fn get_flow_type(
        &self,
        _query_params: &str,
    ) -> CustomResult<payments::CallConnectorAction, errors::ConnectorError> {
        Ok(payments::CallConnectorAction::Trigger)
    }
}
