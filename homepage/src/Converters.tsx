import CodeEditor from "@uiw/react-textarea-code-editor";

import {
  marc_to_json_string,
  json_to_marc_string,
  json_to_yaml_string,
  json_to_toml_string,
  yaml_to_json_string,
  toml_to_json_string,
  format_marc,
} from "marc";
import React from "react";

export function Converters() {
  const [marc, setMarc] = React.useState("");
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
  const updateMarc = React.useCallback((marc: string) => {
    setMarc(marc);
    const json = stringifyError(marc_to_json_string)(marc);
    setJson(json);
    setYaml(stringifyError(json_to_yaml_string)(json));
    setToml(stringifyError(json_to_toml_string)(json));
  }, []);
  React.useEffect(() => {
    updateMarc(
      `
# Map
.materials{"Infinity stones"}."soul affinity" = "fire"
.materials{metal}.reflectivity = 1.0
.materials{metal}.metallic = true
.materials{plastic}.reflectivity = 0.5
.materials{plastic}.conductivity = -1

# Array of objects
.entities[i].material = "metal"
.entities[ ].name = "hero"

.entities[i].name = "monster"
.entities[ ].material = "plastic"

# Multiline string
.description = """
These are common materials.
They are found on Earth.
"""

`.trim()
    );
  }, [updateMarc]);
  const updateJson = (json: string) => {
    setJson(json);
    setMarc(stringifyError(json_to_marc_string)(json));
    setYaml(stringifyError(json_to_yaml_string)(json));
    setToml(stringifyError(json_to_toml_string)(json));
  };
  const updateYaml = (yaml: string) => {
    setYaml(yaml);
    const json = stringifyError(yaml_to_json_string)(yaml);
    setJson(json);
    setMarc(stringifyError(json_to_marc_string)(json));
    setToml(stringifyError(json_to_toml_string)(json));
  };
  const updateToml = (toml: string) => {
    setToml(toml);
    const json = stringifyError(toml_to_json_string)(toml);
    setJson(json);
    setMarc(stringifyError(json_to_marc_string)(json));
    setYaml(stringifyError(json_to_yaml_string)(json));
  };
  return (
    <div style={{ display: "grid", padding: 32 }}>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gridTemplateRows: "40vh 40vh",
          gap: 16,
        }}
      >
        <Editor
          title="MARC"
          value={marc}
          language="python"
          onChange={updateMarc}
          format={format_marc}
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
