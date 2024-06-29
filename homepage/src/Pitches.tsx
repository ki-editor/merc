import "./Pitches.css";
import { omit } from "ramda";
import Markdown from "react-markdown";
import pitch1 from "./pitch1.md?raw";
import pitch2 from "./pitch2.md?raw";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import remarkGfm from "remark-gfm";
import rehypeSlug from "rehype-slug";

export function Pitch1() {
  return <RenderMarkdown markdown={pitch1} />;
}

export function Pitch2() {
  return <RenderMarkdown markdown={pitch2} />;
}

export function RenderMarkdown(props: { markdown: string }) {
  return (
    <div
      style={{
        width: "min(888px, 80vw)",
        padding: "16px 32px",
        maxWidth: 1200,
        justifySelf: "center",
        overflowX: "auto",
      }}
    >
      <Markdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeSlug]}
        components={{
          table(props) {
            return (
              <div style={{ overflowX: "auto" }}>
                <table {...props}>{props.children}</table>
              </div>
            );
          },
          code(props) {
            const { children, className, node, ...rest } = omit(
              ["style", "ref"],
              props
            );
            const match = /language-(\w+)/.exec(className || "");
            return match ? (
              <div>
                {className === "language-ebnf" && ""}
                <SyntaxHighlighter
                  {...rest}
                  children={String(children).replace(/\n$/, "")}
                  language={match[1]}
                />
              </div>
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
