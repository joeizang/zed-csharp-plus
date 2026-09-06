; Outline for the project-file languages.
;
; Shared file: the msbuild, csproj, and slnx language directories each
; need their own query set (Zed resolves a language's queries from its
; own directory only), and all three resolve to this file via symlinks.
; See languages/msbuild/highlights.scm and docs/project-files.md.
;
; The xml grammar is generic, so meaning comes from text predicates on
; element and attribute names (case-sensitive on purpose; MSBuild and
; .slnx element names are PascalCase by convention). The outline is
; curated rather than "every element": it lists the things a project
; file is navigated by, not each property.
;
; Zed joins multiple @name captures with a space, producing entries
; like `Target WriteRevision`, `PackageReference Swashbuckle.AspNetCore`,
; or `Project "src\WebSample.Web\WebSample.Web.csproj"`.

; --- MSBuild: containers --------------------------------------------

((STag (Name) @name)
 (#any-of? @name
   "PropertyGroup" "ItemGroup" "ItemDefinitionGroup" "ImportGroup"
   "Choose" "When" "Otherwise")) @item

; --- MSBuild: named elements (element name + identity attribute) -----
;
; A Target by its Name, a PackageReference by its Include, an Import by
; its Project/Sdk, the root Project by its Sdk. Cross products of the
; two lists are harmless: the combinations that do not occur in real
; files simply never match.

(element
  (STag
    (Name) @name @_tag
    (Attribute
      (Name) @_an
      (AttValue) @name))
  (#any-of? @_tag
    "Project" "Target" "PackageReference" "ProjectReference" "Reference"
    "UsingTask" "Import" "Using" "AssemblyAttribute" "Compile"
    "EmbeddedResource" "Content" "None")
  (#any-of? @_an
    "Name" "Include" "TaskName" "Project" "Sdk")) @item

(element
  (EmptyElemTag
    (Name) @name @_tag
    (Attribute
      (Name) @_an
      (AttValue) @name))
  (#any-of? @_tag
    "Target" "PackageReference" "ProjectReference" "Reference"
    "UsingTask" "Import" "Using" "AssemblyAttribute" "Compile"
    "EmbeddedResource" "Content" "None")
  (#any-of? @_an
    "Name" "Include" "TaskName" "Project" "Sdk")) @item

; --- .slnx: the projects in the solution ----------------------------
;
; Project entries are named by their Path attribute — a required
; attribute for Project elements in the XML solution format, so
; legitimate projects are not silently dropped. "Path" is deliberately
; absent from the MSBuild attribute lists above, so a solution entry
; matches exactly one pattern here.

(element
  (STag
    (Name) @name @_tag
    (Attribute
      (Name) @_an
      (AttValue) @name))
  (#eq? @_tag "Project")
  (#eq? @_an "Path")) @item

(element
  (EmptyElemTag
    (Name) @name @_tag
    (Attribute
      (Name) @_an
      (AttValue) @name))
  (#eq? @_tag "Project")
  (#eq? @_an "Path")) @item
