
/*
Spec MIVOT
https://github.com/ivoa-std/ModelInstanceInVot

parser
https://github.com/ivoa/modelinstanceinvot-code

Groupe de travail sur l'implémentation d'une API astropy
https://github.com/ivoa/modelinstanceinvot-code/wiki
les deux derniers items de Hack-a-thon

wiki API
https://github.com/ivoa/modelinstanceinvot-code/wiki/guideline

service:
https://xcatdb.unistra.fr/xtapdb

RFC:
https://wiki.ivoa.net/twiki/bin/view/IVOA/DataAnnotation

Meas
https://ivoa.net/documents/Meas/20211019/index.html
*/
 
// pos {
//   id: String (ex: pos.eq.main)
//   sys: Option<eq>,
//   ra:  FIELDRef
//   dec: FIELDRef
// }

// pos.err {
//   pos: Option<MODELRef>
//   type: 
//   params (depends on type) 
// }
// Get system associated to error: error.pos.sys