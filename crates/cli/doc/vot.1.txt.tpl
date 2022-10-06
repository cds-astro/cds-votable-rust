vot(1)
=====

Name
----
vot - convert VOTable abck and forth in XML, JSON, TOML and YAML.


Synopsis
--------

*vot* _FORMAT_IN_ _FORMAT_OUT_ _OPTION_

*vot* *--help*

*vot* *--version*

*command* | *vot* [xml|json|toml|yaml] [xml|json|toml|yaml]  [_OPTION_]


FORMAT_IN
---------

_xml_::
  Regular XML VOtable


_json_::
  JSON encoded VOTable

_toml_::

  TOML encoded VOTable

_yaml_::
  YAML encoded VOTable

OPTION
------
_-i_, _--input_::
  Path of the input file (else read from stdin)

_-o_, _--output_ ::
  Path of the output file (else printon stdout)

_-p_, _--pretty_::
  Use pretty print for JSON and TOML



Examples
--------

vot xml json --pretty -i FILE_IN

cat VOT_FILE | vot xml xml


DESCRIPTION
-----------



VERSION
-------
{VERSION}


HOMEPAGE
--------
https://github.com/cds-astro/cds-votable-rust

Please report bugs and feature requests in the issue tracker.


AUTHORS
-------
F.-X. Pineau <francois-xavier.pineau@astro.unistra.fr>


