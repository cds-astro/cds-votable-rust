# `moc-cli` Change Log

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

