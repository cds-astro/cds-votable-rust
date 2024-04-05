#!/bin/bash

in="test_edit.td.xml"


RUST_LOG=warn vot get --in ${in} struct 

# Remove first sub-resource
# Add attributes to the main resource
# Add description to the main resource
# Add coosys with filed_ref to the main resource
# Remove PARAMs
# Add PARAM
# Rename columns
# Remove a link
RUST_LOG=warn vot edit --in ${in} --out-fmt xml-td --streaming \
  -e 'RESOURCE vid=DR1R1 rm' \
  -e 'RESOURCE vid=DR1 set_attrs ID=R1 name=main_resource' \
  -e 'RESOURCE vid=DR1 set_description The main resource containing my super table' \
  -e 'RESOURCE vid=DR1 push_coosys ID=t4-coosys-1 system=eq_FK4 equinox=B1900 
        @push_fieldref ref=RA1900 
	  @@set_content Ref to the RA column @<
        @push_fieldref ref=DE1900 
	  @@set_content Ref to the Declination column @<' \
  -e 'PARAM name=votable-version rm' \
  -e 'PARAM name=-ref rm' \
  -e 'PARAM name=-out.max rm' \
  -e 'PARAM name=queryParameters rm' \
  -e 'TABLE name=I/238A/picat push_param name=hpx_order datatype=unsignedByte value=8 
       @set_description HEALPix order
       @set_values 
         @@set_min value=0  inclusive=true 
	  @set_max value=29 inclusive=true
	 @<
       @push_link href=https://en.wikipedia.org/wiki/HEALPix
         @@set_content General HEALPix info on Wikipedia @<' \
  -e 'FIELD name=recno set_attrs name=RecordNumber' \
  -e 'LINK vid=DR1T1F27l1 rm' \
  -e 'TABLE name=I/238A/picat push_info name=ps value=my post-scriptum @set_content My super post-scriptum' \
  -e 'VOTABLE vid=D push_post_info name=warning value=This table has been modified by using vot-cli'



