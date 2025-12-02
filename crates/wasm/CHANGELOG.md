# `vot-wasm` Change Log

## 0.7.0

Released 2025-12-02

* Better array of array support
* Add support for array of Strings
* Improve code


## 0.6.2

Released 2024-10-18

* Fix boolean value to support any mix of case
* Supports `info` and `post_info` in `TABLE`


## 0.6.1

Released 2024-04-15

* Mainly changes in the cli, no effect on vot-wasm


## 0.6.0

Released 2024-04-05

* Changes in the lib and the cli, no effect on vot-wasm


## 0.5.0

Released 2024-03-11

* Add support for `CDATA` in `TD` tags


## 0.4.0

Released 2024-02-06

* Version and xmlns now mandatory in VOTable (with v1.4 as default)


## 0.3.0

Released 2024-01-12

* Add conversions between TABLEDATA/BINARY/BINARY2
* Supports VOTable 1.5
* Add Mivot parsing (VODML tag)
* For other changes/bug correction, see the `votable` crate changelog


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

