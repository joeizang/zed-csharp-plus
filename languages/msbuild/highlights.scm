; XML declaration
"xml" @keyword

[
  "version"
  "encoding"
  "standalone"
] @property

(EncName) @string.special

(VersionNum) @number

[
  "yes"
  "no"
] @boolean

; Processing instructions
(PI) @embedded

(PI
  (PITarget) @keyword)

; Element declaration
(elementdecl
  "ELEMENT" @keyword
  (Name) @tag)

(contentspec
  (_
    (Name) @property))

"#PCDATA" @type.builtin

[
  "EMPTY"
  "ANY"
] @string.special.symbol

[
  "*"
  "?"
  "+"
] @operator

; Entity declaration
(GEDecl
  "ENTITY" @keyword
  (Name) @constant)

(GEDecl
  (EntityValue) @string)

(NDataDecl
  "NDATA" @keyword
  (Name) @label)

; Parsed entity declaration
(PEDecl
  "ENTITY" @keyword
  "%" @operator
  (Name) @constant)

(PEDecl
  (EntityValue) @string)

; Notation declaration
(NotationDecl
  "NOTATION" @keyword
  (Name) @constant)

(NotationDecl
  (ExternalID
    (SystemLiteral
      (URI) @string.special)))

; Attlist declaration
(AttlistDecl
  "ATTLIST" @keyword
  (Name) @tag)

(AttDef
  (Name) @property)

(AttDef
  (Enumeration
    (Nmtoken) @string))

(DefaultDecl
  (AttValue) @string)

[
  (StringType)
  (TokenizedType)
] @type.builtin

(NotationType
  "NOTATION" @type.builtin)

[
  "#REQUIRED"
  "#IMPLIED"
  "#FIXED"
] @attribute

; Entities
(EntityRef) @constant

((EntityRef) @constant.builtin
  (#any-of? @constant.builtin "&amp;" "&lt;" "&gt;" "&quot;" "&apos;"))

(CharRef) @constant

(PEReference) @constant

; External references
[
  "PUBLIC"
  "SYSTEM"
] @keyword

(PubidLiteral) @string.special

(SystemLiteral
  (URI) @markup.link)

; Processing instructions
(XmlModelPI
  "xml-model" @keyword)

(StyleSheetPI
  "xml-stylesheet" @keyword)

(PseudoAtt
  (Name) @property)

(PseudoAtt
  (PseudoAttValue) @string)

; Doctype declaration
(doctypedecl
  "DOCTYPE" @keyword)

(doctypedecl
  (Name) @type)

; Tags
(STag
  (Name) @tag)

(ETag
  (Name) @tag)

(EmptyElemTag
  (Name) @tag)

; Attributes
(Attribute
  (Name) @property)

(Attribute
  (AttValue) @string)

; Delimiters & punctuation
[
  "<?"
  "?>"
  "<!"
  "]]>"
  "<"
  ">"
  "</"
  "/>"
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
] @punctuation.bracket

[
  "\""
  "'"
] @punctuation.delimiter

[
  ","
  "|"
  "="
] @operator

; Text
(CharData) @markup

(CDSect
  (CDStart) @markup.heading
  (CData) @markup.raw
  "]]>" @markup.heading)

; Misc
(Comment) @comment

(ERROR) @error

; ---------------------------------------------------------------------
; MSBuild vocabulary
;
; The xml grammar is generic: element and attribute names are opaque
; Name tokens, so MSBuild meaning is attached with text predicates.
; Element and attribute names are PascalCase by MSBuild convention;
; the predicates are case-sensitive on purpose.
;
; This file is shared: the msbuild, csproj, and slnx language
; directories each need their own query set (Zed resolves a language's
; queries from its own directory only), and all three resolve to this
; file via symlinks. So the vocabulary for every dialect lives here as
; disjoint sections — no element name appears in two lists, or a node
; would match (and snapshot) twice. See docs/project-files.md.

; Structural containers and control flow read as keywords
((STag (Name) @keyword)
 (#any-of? @keyword
   "Project" "PropertyGroup" "ItemGroup" "ItemDefinitionGroup" "ImportGroup"
   "Target" "Choose" "When" "Otherwise" "Import" "Using" "UsingTask"
   "ProjectExtensions" "Sdk"))

((EmptyElemTag (Name) @keyword)
 (#any-of? @keyword
   "Import" "Using" "UsingTask" "Sdk"))

; Task invocations inside a Target read as function calls
(element
  (STag
    (Name) @_target)
  (content
    (element
      (STag (Name) @function.call)))
  (#eq? @_target "Target"))

(element
  (STag
    (Name) @_target)
  (content
    (element
      (EmptyElemTag (Name) @function.call)))
  (#eq? @_target "Target"))

; Well-known controlling attributes
((Attribute (Name) @attribute)
 (#any-of? @attribute
   "Condition" "Include" "Exclude" "Remove" "Update" "Label" "Name"
   "BeforeTargets" "AfterTargets" "DependsOnTargets" "Inputs" "Outputs"
   "Sdk" "Version" "PrivateAssets" "ContinueOnError" "TreatAsLocalProperty"))

; ---------------------------------------------------------------------
; .slnx (XML solution format) vocabulary
;
; Kept in this shared file for the same reason as the MSBuild section
; above. "Project" and "Target" are already keywords via the MSBuild
; list, so only the solution-only names appear here.

; Solution structure
((STag (Name) @keyword)
 (#any-of? @keyword "Solution" "Deployment"))

((EmptyElemTag (Name) @keyword)
 (#any-of? @keyword "Deployment"))

; Well-known solution attributes ("Name" is covered by the MSBuild list)
((Attribute (Name) @attribute)
 (#any-of? @attribute "Path" "Configuration" "Platform"))
