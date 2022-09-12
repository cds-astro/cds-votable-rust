import os
import xmltodict
import json
import numpy
from lxml import etree

class MyEncoder(json.JSONEncoder):

    def default(self, obj):
        if isinstance(obj, numpy.integer):
            return int(obj)
        elif isinstance(obj, numpy.floating):
            return float(obj)
        elif isinstance(obj, numpy.ndarray):
            return obj.tolist()
        else:
            return super(MyEncoder, self).default(obj)

data_path = os.path.dirname(os.path.realpath(__file__))

xml_block = etree.parse(
    os.path.join(data_path, "votable_to_json.xml"))
raw_json = xmltodict.parse(etree.tostring(xml_block))
pretty_json = json.dumps(raw_json,
                          indent=2,
                          cls=MyEncoder)
print(pretty_json)

with open(os.path.join(data_path, "votable_to_json.json"), 'w') as file:
                file.write(json.dumps(raw_json, indent=2))

