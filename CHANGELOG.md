# `votable` Change Log

## 0.6.0

Released 2024-04-05

### Changed

* ⚠️  BREAKING: the tag is now in the `VOTableTrait` instead of the `QuickXmlReadWrite` trait 
* ⚠️  BREAKING: extra attribute names (allowed in some Tags in the lib, but not in the standard) 
  must contains ':'. If not, a prefix 'extra:' is automatically addded by the API (not when deserializing)
* ⚠️  BREAKING: update the MIVOT visitor to consistent with the VOTable visitor
* Add `for_each_attribute` in MIVOT tags
* Add setters
* Add documentation
* Internal changes: 
    + add trait `VOTableElement` for more genericity and to lighten the role of the `QuickXMLReadWrite` trait 
    + add markers trait to distinguish between several `VOTableElement` patterns
    + remove macros


## 0.5.0

Released 2024-03-11

### Added

* Add genericity and remove (some) duplicated code
* Add support for `CDATA` in `TD` tags
* See `votcli` changelog


## 0.4.0

Released 2024-02-06

### Added

* Enrich API with elements such as `push_elem` or `set_xx_by_ref` and re-export.
* Provide with a MIVOT `DoNothing` visitor 
* Provide with a `VOTableVisitor` trait and `visitor` methods on the ful VOTable
* Add methods to merge together two `Fields`

### Changed

* `Version` and `xmlns` are now mandatory in VOTable (with v1.4 as edfault)
* Add `xmlns:xsi` and `xsi:schemaLocation` in VOTable optional attributes (instead of extra)
* Add logger to control `stderr` messages


## 0.3.0

Released 2024-01-12

### Added
 
* Add conversions between TABLEDATA/BINARY/BINARY2
* Add support for VOTable 1.5: `refposition`, `FIELDref` and `PARAMref` allowed in CooSys
* Add `SimpleVOTableRowIterator` with `OwnedTabledataRowIterator` and `OwnedBinary1or2RowIterator` 
   to make external parsers taking charge of parsing rows
* Add methods `get_first_table` and `get_first_table_mut` in votable
* More attributes/sub-elements are now public
* Add Mivot support with feature "mivot"
* Add PartialEq implementation
* Add `ensures_consistency` after JSON/TOML/YAML deserialization to ensure that
  type in memory are coherent with the table schema

### Changed

* ⚠️  BREAKING: add a `ResourceSubElem` structure in `Resource` to pack together
  LINKS, RESOURCE or TABLE, INFO (the choice in the VOTable xsd, the figure is missleading)
* ⚠️  BREAKING: Arraysize no more a String but a enum

### Bug correction

* Better handling of arrays (please provide us with examples so we can test and debug)
* Fix unicode char bug
* Fix breaking change introduced in `serde.__private`
* Fix "hint" --> "hints" in LINK attribute "content-role"

## 0.2.3

Released 2023-05-01

* Accept CDATA in 'Info', 'Desciption', 'Link', 'ParamRef' and 'FieldRef' content
* Accepth empty 'precision' and 'width' attributes

## 0.2.2

Released 2023-05-01

* Accept VOTables 1.0
* Add the deprecated "DEFINITIONS" tag


## 0.2.1

Released 2023-04-25

* Accept VOTables 1.1 and 1.2
* Fix error while parsing PARAM
* Fix error with empty fields (?) 


## 0.2.0

Released 2023-03-30

### Added

* class `VOTableIterator` to iterate externally on a VOTable
  tables and table rows

### Bug correction

* Now works with namespaces (simply ignoring them)
* Support PARAM containing sub-elements in GROUP


## 0.1.1-alpha

Released 2022-10-10.

### Bug correction

* Fix error on "boolean" datatype FIELDs
* Fix "boolean" datatype parsing in tabledata


## 0.1.0-alpha

Released 2022-10-06.

