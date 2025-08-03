# A simple json parser
A simple json parser written in pure rust without any dependencies. It has a pretty-printer which prints the parsed json value to s-expression.

 # Rules for converting json object to s-expression 
 Json Grammer is transformed according to the following rules
 - `dict = "(" [pair (" " pair)*] ")"`
 - `pair = "(" key " " value ")"`
 - `list = "(" [value (" "value)*] ")"`
 - The keys are printed without quotes.
 - The atoms `(string | number | "true" | "false" | "null")` are printed as is.
