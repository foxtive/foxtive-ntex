# Foxtive-Ntex Changelog
Foxtive-Ntex changelog file 

------

### 0.17.0 (2025-08-05)
* bump(foxtive): to version 0.15

### 0.16.0 (2025-06-19)
* bump(foxtive): to version 0.14

### 0.15.2 (2025-06-14)
* bump(foxtive): to version 0.13.2

### 0.15.1 (2025-06-09)
* bump(ntex-cors): to version 2.1.0

### 0.15.0 (2025-06-09)
* bump(ntex): to version 2.15
* bump(ntex-files): to version 2.1.0
* bump(ntex-cors): to version 2.1.0

### 0.14.0 (2025-06-08)
* feat(multipart): add uuid support
* feat(data-input): add 'post', 'post_or', 'post_opt' methods

### 0.13.0 (2025-06-08)
* bump(foxtive): to version 0.13

### 0.12.2 (2025-06-08)
* bump(foxtive-ntex-multipart): to version 0.x.x

### 0.12.1 (2025-06-05)
* bump(foxtive-ntex-multipart): to version 0.3.0

### 0.12.0 (2025-06-25)
* feat(DeJsonBody): impl Deref & DerefMut
* bump(ntex): to version 2.14.0

### 0.11.0 (2025-06-14)
* feat(extractors): add DeJsonBody
* bump(foxtive): to version 0.10

### 0.10.0 (2025-05-23)
* bump(foxtive): to version 0.10

### 0.9.0 (2025-05-07)
* feat(string-body): extractor for string body
* feat(byte-body): extractor for byte body
* feat(jwt-auth-token): extractor for bearer token extraction from header and decoding
* deprecate(json-body.raw()): use body() instead

### 0.8.0 (2025-04-20)
* bump(foxtive): to version 0.8

### 0.7.0 (2025-04-20)
* bump(foxtive): to version 0.7

### 0.6.2 (2025-04-02)
* feat(message): impl AppMessageExt for Result<AppMessage, BlockingError<AppMessage>>

### 0.6.1 (2025-04-01)
* feat(message): log errors to console before sending them as response

### 0.6.0 (2025-03-28)
* feat(message): added AppMessageExt to provide .respond() for AppMessage

### 0.5.6 (2025-03-28)
* fix(client-info): use HttpError as error type

### 0.5.5 (2025-03-27)
* fix(json-body): ensure bad request is returned when there's a deserialization error

### 0.5.4 (2025-03-27)
* fix(json-body): return bad request error when deserialization fails

### 0.5.3 (2025-03-27)
* fix(http-error): convert and return proper error response

### 0.5.2 (2025-03-22)
* fix(message): return json-formatted error message

### 0.5.1 (2025-03-17)
* bump(foxtive): to fix app message issue

### 0.5.0 (2025-03-16)
* refactor(features): remove unused
* feat(error): impl from validator::ValidationErrors & foxtive_ntex_multipart::MultipartError

### 0.4.0 (2025-03-16)
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

