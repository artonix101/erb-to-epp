# ERB to EPP Converter

A CLI tool written in rust that converts embedded ruby (.erb) templates into embedded puppet (.epp) compatible syntax by:
 - Replacing @ with $ inside template tags
 - Converting if, elsif, else, and end into proper .epp blocks with curly brackets
 - Converting simple .each loops into proper .epp
 - Converting complex .each loops that iterate through empty hashes (with || {}) by adding an if
 - Preserving optional whitespace trimmers (<%- and -%>)
 - Leaving @ outside tags (like in email addresses) unchanged
 - Converting versioncmp fn into proper .epp
 - Add missing $ to variables inside tags

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

<% @features.each do | f | %>
  <%= f %>
<% end %>
<%- @features.each do | f, g, h | -%>
  <%= f %><%= g['something'] %><%= h -%>
<% end %>

<% if scope.function_versioncmp([@version, '1.0']) < 0 %>
  Do Something
<% end %>

<%- (@var['thing1']['thing2']['thing3'] || {}).each do | x | -%>
    Thing =<%= x['thing'] %>
<% end -%>

An Email: test@gmx.de
```
output.epp
```
<%- if $x { -%>
  Hello <%= $name %>
<% } else if $y { %>
  Hello <%= $other_name %>
<%- } else { -%>
  No variable
<%- } -%>

<% $features.each | $f | { %>
  <%= $f %>
<% } %>
<%- $features.each | $f, $g, $h | { -%>
  <%= $f %><%= $g['something'] %><%= $h -%>
<% } %>

<% if versioncmp($version, '1.0') < 0 { %>
  Do Something
<% } %>

<%- if ('thing1' in $var) and ('thing2' in $var['thing1']) and ('thing3' in $var['thing1']['thing2']) and ($var['thing1']['thing2']['thing3'] =~ Array) { -%>
<%- $var['thing1']['thing2']['thing3'].each | $x | { -%>
    Thing =<%= $x['thing'] %>
<% } -%>
<%- } -%>

An Email: test@gmx.de
```

### ToDo

 - add $ to vars inside loops as seen in example above (f,g,h) ✅
 - add creation of parameter tags like <%- | Hash $hash | -%> to beginning of new epp file
 - solve problem of which parameter tags should be added and what datatype they get based on what is in the template. Not so easy (without knowing the manifest or hiera there is no way of knowing what the datatypes are). This could be done with manual input from user like this:

```
Found following parameters in input.erb:
@x, @name, @othername, @features, @version, @var
Is this correct? If not please specify which of these to eliminate (separated by commas if more than one):
Done.
Specify Datatypes of parameters for the tags (check your manifest or hiera, i.e. hash, string, boolean)
$x:
$name:
$othername:
$features:
$version:
$var:

Done.
```
