import * as React from "react";
import Markdown from "react-markdown";
import pitch1 from "./pitch1.md?raw";
import pitch2 from "./pitch2.md?raw";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";

export function Pitch1() {
  return <RenderMarkdown markdown={pitch1} />;
}

export function Pitch2() {
  return <RenderMarkdown markdown={pitch2} />;
}

export function RenderMarkdown(props: { markdown: string }) {
  return (
    <div
      style={{ padding: "16px 32px", maxWidth: 1200, justifySelf: "center" }}
    >
      <Markdown
        components={{
          code(props) {
            const { children, className, node, style, ref, ...rest } = props;
            const match = /language-(\w+)/.exec(className || "");
            return match ? (
              <SyntaxHighlighter
                {...rest}
                children={String(children).replace(/\n$/, "")}
                language={match[1]}
              />
            ) : (
              <code {...rest}>{children}</code>
            );
          },
        }}
      >
        {props.markdown}
      </Markdown>
    </div>
  );
}
