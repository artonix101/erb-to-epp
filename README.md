# ERB to EPP Converter

A CLI tool written in rust that converts embedded ruby (.erb) templates into embedded puppet (.epp) compatible syntax by:
 - Replacing @ with $ inside template tags
 - Converting if, elsif, else, and end into proper .epp blocks with curly brackets
 - Preserving optional whitespace trimmers (<%- and -%>)
 - Leaving @ outside tags (like in email addresses) unchanged

### Usage

```
#convert and print to stdout
./erb-to-epp input.erb
```
```
#convert and save to an output file
:/erb-to-epp input.erb output.epp
```

### Example

input.erb
```
<%- if @x -%>
  Hello <%= @name %>
<% elsif @y %>
  Hello <%= @other_name %>
<%- else -%>
  No variable
<%- end -%>
An Email: test@gmx.de
```
output.epp
```
<%- if $x { -%>
  Hello <%= $name %>
<% } elsif $y { %>
  Hello <%= $other_name %>
<%- } else { -%>
  No variable
<%- } -%>
An Email: test@gmx.de
``
