"use strict";(self.webpackChunkweb_app_certos=self.webpackChunkweb_app_certos||[]).push([[299],{99299:function(e,t,n){n.r(t);var u=n(68457),r=n(81349),a=(n(29867),n(28544),Object.defineProperty),l=function(e,t){return a(e,"name",{value:t,configurable:!0})};function i(e,t){var n,u,r=e.levels,a=e.indentLevel;return((r&&0!==r.length?r[r.length-1]-((null===(n=this.electricInput)||void 0===n?void 0:n.test(t))?1:0):a)||0)*((null===(u=this.config)||void 0===u?void 0:u.indentUnit)||0)}u.C.defineMode("graphql-variables",(function(e){var t=(0,r.o)({eatWhitespace:function(e){return e.eatSpace()},lexRules:c,parseRules:s,editorConfig:{tabSize:e.tabSize}});return{config:e,startState:t.startState,token:t.token,indent:i,electricInput:/^\s*[}\]]/,fold:"brace",closeBrackets:{pairs:'[]{}""',explode:"[]{}"}}})),l(i,"indent");var c={Punctuation:/^\[|]|\{|\}|:|,/,Number:/^-?(?:0|(?:[1-9][0-9]*))(?:\.[0-9]*)?(?:[eE][+-]?[0-9]+)?/,String:/^"(?:[^"\\]|\\(?:"|\/|\\|b|f|n|r|t|u[0-9a-fA-F]{4}))*"?/,Keyword:/^true|false|null/},s={Document:[(0,r.p)("{"),(0,r.l)("Variable",(0,r.b)((0,r.p)(","))),(0,r.p)("}")],Variable:[o("variable"),(0,r.p)(":"),"Value"],Value:function(e){switch(e.kind){case"Number":return"NumberValue";case"String":return"StringValue";case"Punctuation":switch(e.value){case"[":return"ListValue";case"{":return"ObjectValue"}return null;case"Keyword":switch(e.value){case"true":case"false":return"BooleanValue";case"null":return"NullValue"}return null}},NumberValue:[(0,r.t)("Number","number")],StringValue:[(0,r.t)("String","string")],BooleanValue:[(0,r.t)("Keyword","builtin")],NullValue:[(0,r.t)("Keyword","keyword")],ListValue:[(0,r.p)("["),(0,r.l)("Value",(0,r.b)((0,r.p)(","))),(0,r.p)("]")],ObjectValue:[(0,r.p)("{"),(0,r.l)("ObjectField",(0,r.b)((0,r.p)(","))),(0,r.p)("}")],ObjectField:[o("attribute"),(0,r.p)(":"),"Value"]};function o(e){return{style:e,match:function(e){return"String"===e.kind},update:function(e,t){e.name=t.value.slice(1,-1)}}}l(o,"namedKey")}}]);
//# sourceMappingURL=299.4f82f194.chunk.js.map