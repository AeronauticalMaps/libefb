# AIXM from XSD generator

Navigation data are provided by authorities in the AIXM format. It is
specified by following XSD files:

- https://www.aixm.aero/schema/5.2/5.2.0/AIXM_Features.xsd
- https://www.aixm.aero/schema/5.2/5.2.0/message/AIXM_BasicMessage.xsd

The Rust crate `xsd-parser` is capable of parsing XSD files and
generate Rust types from it. It has a `web-resolver` feature that
enables resolving schema provided as URL. The parser is split multiple
stages and can be configured using a builder pattern. Add the two XSD
schema provided above to the parser and configure it to use the
`Resolver::Web`.

The AIXM model has some types that need special handling since the
generator would generate invalid Rust code from them. Some enums are
named `+` or `-` which needs to be translated to e.g. `Plus` or `Minus`.

The author of the `xsd-parser` crate has a feature branch
`https://github.com/Bergmann89/xsd-parser/tree/feature/aixm` where he
writes a builder to parse AIXM files
`https://github.com/Bergmann89/xsd-parser/blob/feature/aixm/examples/simple.rs`. The
code uses local files opposed to the approach we want to implement and
writes all code into one file. However, it gives an idea on how to add
stages to the parser to fix the types that would result in invalid
Rust code. The issue is, that the feature branch is based on an
earlier version and there were breaking API changes compared to the
latest version. You need to find the difference to implement this
custom name parsing for the latest version of the crate.

The code returned by the `generate` function is a `TokenStream`. This
code SHOULDN'T be written into on file. It MUST be split into modules.
