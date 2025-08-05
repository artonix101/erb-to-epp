use regex::Regex;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <inputfile> [outputfile]", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];

    //read input file
    let input_content = fs::read_to_string(input_file)
        .unwrap_or_else(|_| {
            eprintln!("Error: Could not read file {}", input_file);
            process::exit(1);
        });

    //convert content
    let output_content = convert_code(&input_content);

    //write or print output
    if args.len() == 3 {
        let output_file = &args[2];
        if let Err(e) = fs::write(output_file, output_content) {
            eprintln!("Error: Could not write to {}: {}", output_file, e);
            process::exit(1);
        }
    } else {
        println!("{}", output_content);
    }
}

fn convert_code(input: &str) -> String {
    //regex patterns
    let tag_re = Regex::new(r"<%[\-=]?\s*.*?%>").unwrap(); //tags
    let var_re = Regex::new(r"@([a-zA-Z_]\w*)").unwrap(); //variables
    let if_re = Regex::new(r"<%(?P<open_dash>-?)\s*if\s+(?P<cond>.*?)\s*(?P<close_dash>-?)%>").unwrap(); //if
    let end_re = Regex::new(r"<%(?P<open_dash>-?)\s*end\s*(?P<close_dash>-?)%>").unwrap(); //end
    let elsif_re = Regex::new(r"<%(?P<open_dash>-?)\s*elsif\s+(?P<cond>.*?)\s*(?P<close_dash>-?)%>").unwrap(); //elsif
    let else_re = Regex::new(r"<%(?P<open_dash>-?)\s*else\s*(?P<close_dash>-?)%>").unwrap(); //else
    let each_re_1= Regex::new(r"<%(?P<open_dash>-?)\s*\s+(?P<cond>.*?)\.each\s+do\s+\|\s*(?P<each_args>.*?)\s*\|\s*(?P<close_dash>-?)%>").unwrap(); //each
    let each_args_re = Regex::new(r"\|\s*([a-zA-Z_]\w*(?:\s*,\s*[a-zA-Z_]\w*)*)\s*\|").unwrap(); //each_args
    let var_no_dollar_re = Regex::new(r"<%=\s*(?P<expr>\w+(?:\[[^\]]+\]|\.\w+)*)\s*(?P<close_dash>-?)%>").unwrap();  //vars without $
    let each_re_2 = Regex::new(r"<%(?P<open_dash>-?)\s*\(\s*(?P<cond>\$[^\)]+?)\s*\|\|\s*\{\}\s*\)\.each\s+\|\s*(?P<each_args>[^|]+?)\s*\|\s*\{\s*(?P<close_dash>-?)%>").unwrap(); //second each iteration
    let loop_end_re = Regex::new(r"(?m)(<%(?P<open_dash>-?)\s*\}\s*(?P<close_dash>-?)%>)").unwrap(); //end of loop tag
    let scope_fn_re = Regex::new(r"\bscope\.function_").unwrap(); //scope.function_
    let versioncmp_brackets_re = Regex::new(r"(versioncmp\s*\()\s*\[\s*(.*?)\s*\]\s*(\))").unwrap(); //square bracket insode versioncmp

    //process tags
    let mut result = input.to_string();
    let mut transformed_loops = 0; //count tranformed loops
    result = tag_re
        .replace_all(&result, |caps: &regex::Captures| {
            let mut tag = caps[0].to_string();
            // Replace all @var with $var
            tag = var_re
                .replace_all(&tag, |c: &regex::Captures| format!("${}", &c[1]))
                .to_string();
            //convert <% if ... %> to <% if ... { %>
            tag = if_re.replace_all(&tag, "<%$open_dash if $cond { $close_dash%>").to_string();
            //convert <% end %> to <% } %>
            tag = end_re.replace_all(&tag, "<%$open_dash } $close_dash%>").to_string();
            //convert <% elsif ... %> to <% } elsif { %>
            tag = elsif_re.replace_all(&tag, "<%$open_dash } else if $cond { $close_dash%>").to_string();
            //convert <% else %> to <% } else { %>
            tag = else_re.replace_all(&tag, "<%$open_dash } else { $close_dash%>").to_string();
            //convert <% ....each do | f,g,... | %> to <% ....each | f,g,... | { %>
            tag = each_re_1.replace_all(&tag, "<%$open_dash $cond.each | $each_args | { $close_dash%>").to_string();
            //convert | f,g,... | to | $f,$g,... | in loops
            tag = each_args_re.replace_all(&tag, |caps: &regex::Captures| {
                let vars = &caps[1];
                let replaced: Vec<String> = vars
                    .split(',')
                    .map(|v| format!("${}", v.trim()))
                    .collect();
                format!("| {} |", replaced.join(", "))
            }).to_string();
            // add missing $ inside tags
            tag = var_no_dollar_re.replace_all(&tag, |caps: &regex::Captures| {
                let expr = &caps["expr"];
                //let var = &caps[1];
                let close_dash = caps.name("close_dash").map_or("", |m| m.as_str());
                let root_var = expr.split(|c| c == '[' || c == '.').next().unwrap_or("");
                if root_var.starts_with('$') {
                    caps[0].to_string()
                } else {
                    let rewritten_expr = expr.replacen(root_var, &format!("${}", root_var), 1);
                    format!("<%= {} {}%>", rewritten_expr, close_dash)
                }
            }).to_string();
            // add if to before each fn
            tag = each_re_2.replace_all(&tag, |caps: &regex::Captures| {
                transformed_loops += 1;
                let open_dash = caps.name("open_dash").map_or("", |m| m.as_str());
                let close_dash = caps.name("close_dash").map_or("", |m| m.as_str());
                let cond = &caps["cond"];
                let each_args = &caps["each_args"];
                //extract keys from the expression like $vars['a']['b']['c']
                let key_re = Regex::new(r"\['(.*?)'\]").unwrap();
                let keys: Vec<_> = key_re
                    .captures_iter(cond)
                    .map(|c| c[1].to_string())
                    .collect();
                let root_var = cond
                    .split('[')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                //build conditional checks: 'key' in previous_path
                let mut path = root_var.clone();
                let mut checks = Vec::new();
                for key in &keys[..keys.len().saturating_sub(1)] {
                    checks.push(format!("('{}' in {})", key, path));
                    path += &format!("[\'{}\']", key);
                }
                //final path for type checki
                if let Some(last_key) = keys.last() {
                    let parent_path = path.clone();
                    checks.push(format!("('{}' in {})", last_key, parent_path));
                    path += &format!("[\'{}\']", last_key);
                    checks.push(format!("({} =~ Array)", path));
                }
                //join if by "and"
                let condition = checks.join(" and ");
                format!(
                    "<%- if {} {{ -%>\n<%{} {}.each | {} | {{ {}%>",
                    condition,
                    open_dash, path, each_args.trim(), close_dash
                )
            }).to_string();
            //add tag to end of .each loop
            let mut count = 0;
            tag = loop_end_re.replace_all(&tag, |caps: &regex::Captures| {
                let mut result = caps[0].to_string();
                if count < transformed_loops {
                    result.push_str("\n<%- } -%>");
                    count += 1;
                }
                result
            }).to_string();
            //remove scope.function_
            tag = scope_fn_re.replace_all(&tag, "").to_string();
            //remove square brackets inside versioncmp
            tag = versioncmp_brackets_re.replace_all(&tag, "$1$2$3").to_string();
            tag //output tag
        }).to_string();
    result
}
