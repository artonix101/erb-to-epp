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

    // Read input file
    let input_content = fs::read_to_string(input_file)
        .unwrap_or_else(|_| {
            eprintln!("Error: Could not read file {}", input_file);
            process::exit(1);
        });

    // Convert content
    let output_content = convert_code(&input_content);

    // Write or print output
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
    // Regex patterns
    let tag_re = Regex::new(r"<%[\-=]?\s*.*?%>").unwrap(); //tags
    let var_re = Regex::new(r"@([a-zA-Z_]\w*)").unwrap(); //variables
    let if_re = Regex::new(r"<%(?P<open_dash>-?)\s*if\s+(?P<cond>.*?)\s*(?P<close_dash>-?)%>").unwrap(); //if
    let end_re = Regex::new(r"<%(?P<open_dash>-?)\s*end\s*(?P<close_dash>-?)%>").unwrap(); //end
    let elsif_re = Regex::new(r"<%(?P<open_dash>-?)\s*elsif\s+(?P<cond>.*?)\s*(?P<close_dash>-?)%>").unwrap(); //elsif
    let else_re = Regex::new(r"<%(?P<open_dash>-?)\s*else\s*(?P<close_dash>-?)%>").unwrap(); //else

    // Process tags
    let mut result = input.to_string();
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

            tag
        }).to_string();
    result
}
