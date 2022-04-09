# Changelog

## [0.8.0] - 2022-04-08
### Added
- Models grammar

## [0.7.1] - 2021-03-30
### Fixed
- Title and version available in `api-spec.json` files too

## [0.7.0] - 2021-03-30
### Added
- Support for keywords: `title` and `version` in `.api` file
- Basic swagger generation (`openapi.json`)

## [0.6.0] - 2021-03-06
### Added
- Extending language to support `tags` list attribute for api operations 
- Extending language to support groups for: headers, parameters, query strings
  and status codes
- Examples for APIs are an array to allow multiple entries
- `.api` File does not require return character at the end
### Fixed
- Blank status codes in `api.json`

## [0.5.0]
### Added
- File watcher to continuously listen for changes in source file

## [0.4.0] - 2021-02-10
### Added
- Tab character can be used for indentation

## [0.3.0]
## Added
- Null values are not serialized
- Program prints version
- Support to attach examples via json file

## [0.2.0] - 2021-02-01
### Added
- Support for field `operation`
### Fixed
- Data type for query, path param and headers