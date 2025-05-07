# Foxtive-Ntex Changelog
Foxtive-Ntex changelog file 

------

### 0.9.0 (2024-05-07)
* feat(string-body): extractor for string body
* feat(byte-body): extractor for byte body
* feat(jwt-auth-token): extractor for bearer token extraction from header and decoding
* deprecate(json-body.raw()): use body() instead

### 0.8.0 (2024-04-20)
* bump(foxtive): to version 0.8

### 0.7.0 (2024-04-20)
* bump(foxtive): to version 0.7

### 0.6.2 (2024-04-02)
* feat(message): impl AppMessageExt for Result<AppMessage, BlockingError<AppMessage>>

### 0.6.1 (2024-04-01)
* feat(message): log errors to console before sending them as response

### 0.6.0 (2024-03-28)
* feat(message): added AppMessageExt to provide .respond() for AppMessage

### 0.5.6 (2024-03-28)
* fix(client-info): use HttpError as error type

### 0.5.5 (2024-03-27)
* fix(json-body): ensure bad request is returned when there's a deserialization error

### 0.5.4 (2024-03-27)
* fix(json-body): return bad request error when deserialization fails

### 0.5.3 (2024-03-27)
* fix(http-error): convert and return proper error response

### 0.5.2 (2024-03-22)
* fix(message): return json-formatted error message

### 0.5.1 (2024-03-17)
* bump(foxtive): to fix app message issue

### 0.5.0 (2024-03-16)
* refactor(features): remove unused
* feat(error): impl from validator::ValidationErrors & foxtive_ntex_multipart::MultipartError

### 0.4.0 (2024-03-16)
* fix: renamed 'FOXTIVE_WEB' to 'FOXTIVE_NTEX'

### 0.3.0
* feat(http): introduced HttpError to handle crate-level error

### 0.2.1
* fix(result): 'is_empty' should return false when value is present
* fix(responder): should return appropriate status code

### 0.2.0
* refactor(defs): renamed to 'ext'
* impl From<Error> for ResponseError
* HttpResult is now Result<ntex::web::HttpResponse, ResponseError>

