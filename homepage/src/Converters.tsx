import CodeEditor from "@uiw/react-textarea-code-editor";
import { useWindowSize } from "@uidotdev/usehooks";

import {
  merc_to_json_string,
  json_to_merc_string,
  json_to_yaml_string,
  json_to_toml_string,
  yaml_to_json_string,
  toml_to_json_string,
  format_merc,
} from "merc";
import React from "react";

export function Converters() {
  const [merc, setMerc] = React.useState("");
  const [json, setJson] = React.useState("");
  const [yaml, setYaml] = React.useState("");
  const [toml, setToml] = React.useState("");
  const stringifyError = (f: (s: string) => string) => {
    return (s: string) => {
      try {
        return f(s);
      } catch (error) {
        return (error as Error).toString();
      }
    };
  };
  const updateMerc = React.useCallback((merc: string) => {
    setMerc(merc);
    const json = stringifyError(merc_to_json_string)(merc);
    setJson(json);
    setYaml(stringifyError(json_to_yaml_string)(json));
    setToml(stringifyError(json_to_toml_string)(json));
  }, []);
  React.useEffect(() => {
    updateMerc(
      `
# Numbers and booleans are identical to that of JSON
.pic = 3.767612653
.sextillion = -6.02e+23
.is_merc = true


# Array of objects (using explicit keys)
# These user-defined keys are solely to construct the array
# They are not consumable by application code
.languages[javascript].extension = "js"
.languages[javascript].server = "typescript-language-server"
.languages[rust].extension = "rs"
.languages[rust].server = "typescript-language-server"


# Scalar array (using implicit keys +)
# You can think of + as auto-incremented integer
.excludes[+] = "node_modules/"
.excludes[+] = "dist/"
.excludes[+] = "target/" 

# Map
# Map and object are identical implementation-wise
# But map keys signify to the reader that they are user-defined
# instead of schema-defined
.dependencies{"@types/react-markdown"} = "~0.2.3"
.dependencies{graphql} = "1.2.3"
.dependencies{react}.name = "^0.1.0"

# Singleline Escaped String
.poem = "Lorem\\tIpsum '''is the best'''"

# Multiline-able Escaped string
.escaped-multiline = """
I must start and end with a newline.
Otherwise it would be an '''error'''.
The first and last newline will be omitted in the constructed string.
"""

# Singleline Raw String
.path = '\\n is not escaped'

# Multiline raw string
.description = '''

'Hello there!'
These are common materials.
They are stored in C:\\SolarSystem:\\Earth

'''

`.trim()
    );
  }, [updateMerc]);
  const updateJson = (json: string) => {
    setJson(json);
    setMerc(stringifyError(json_to_merc_string)(json));
    setYaml(stringifyError(json_to_yaml_string)(json));
    setToml(stringifyError(json_to_toml_string)(json));
  };
  const updateYaml = (yaml: string) => {
    setYaml(yaml);
    const json = stringifyError(yaml_to_json_string)(yaml);
    setJson(json);
    setMerc(stringifyError(json_to_merc_string)(json));
    setToml(stringifyError(json_to_toml_string)(json));
  };
  const updateToml = (toml: string) => {
    setToml(toml);
    const json = stringifyError(toml_to_json_string)(toml);
    setJson(json);
    setMerc(stringifyError(json_to_merc_string)(json));
    setYaml(stringifyError(json_to_yaml_string)(json));
  };
  const windowSize = useWindowSize();
  const largeScreen = (windowSize.width ?? 0) > 1000;
  return (
    <div style={{ display: "grid", padding: 32 }}>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: largeScreen ? "1fr 1fr" : "1fr",
          gridTemplateRows: largeScreen ? "repeat(2, 50vh)" : "repeat(4, 50vh)",
          gap: 16,
        }}
      >
        <Editor
          title="MERC"
          value={merc}
          language="python"
          onChange={updateMerc}
          format={format_merc}
        />
        <Editor
          title="JSON"
          value={json}
          language="json"
          onChange={updateJson}
        />
        <Editor
          title="YAML"
          value={yaml}
          language="yaml"
          onChange={updateYaml}
        />
        <Editor
          title="TOML"
          value={toml}
          language="toml"
          onChange={updateToml}
        />
      </div>
    </div>
  );
}

const Editor = (props: {
  title: string;
  language: string;
  value: string;
  onChange: (value: string) => void;
  format?: (value: string) => string;
}) => {
  const { format } = props;
  return (
    <div
      style={{
        display: "grid",
        gridTemplateRows: "auto 1fr",
        gap: 8,
      }}
    >
      <div
        style={{
          display: "grid",
          gridAutoFlow: "column",
          alignItems: "end",
          gap: 4,
          height: 40,
        }}
      >
        <div className="title">{props.title}</div>
        {format && (
          <button
            style={{
              color: "black",
              backgroundColor: "lightblue",
              fontWeight: "bold",
              width: 200,
              justifySelf: "end",
            }}
            onClick={() => props.onChange(format(props.value))}
          >
            Format
          </button>
        )}
      </div>
      <div style={{ display: "grid", overflow: "auto" }}>
        <div style={{ display: "grid" }}>
          <CodeEditor
            value={props.value}
            language={props.language}
            onChange={(event) => props.onChange(event.target.value)}
            padding={16}
            style={{
              width: "100%",
              backgroundColor: "#f5f5f5",
              fontSize: 16,
              fontFamily:
                "ui-monospace,SFMono-Regular,SF Mono,Consolas,Liberation Mono,Menlo,monospace",
            }}
          />
        </div>
      </div>
    </div>
  );
};
