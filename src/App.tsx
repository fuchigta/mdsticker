import { Fragment, useState } from "react";
import "./App.css";
import Markdown from "react-markdown";
import {Prism as SyntaxHighlighter} from 'react-syntax-highlighter'
import {solarizedDarkAtom} from 'react-syntax-highlighter/dist/esm/styles/prism'

function App() {
  const [editting, setEditting] = useState(false);
  const [markdown, setMarkdown] = useState("");

  return (
    <div className="container">
      <header>
        <button
          onClick={(e) => {
            e.preventDefault();
            setEditting(!editting);
          }}
        >
          {editting ? "DONE" : "EDIT"}
        </button>
      </header>
      <main>
        <Fragment>
          {editting ? (
            <textarea
              value={markdown}
              onChange={(e) => {
                e.preventDefault();
                setMarkdown(e.target.value);
              }}
            ></textarea>
          ) : (
            <div className="markdown">
              <Markdown
              children={markdown}
              components={{
                code(props) {
                  const {children, className, node, ref, ...rest} = props
                  const match = /language-(\w+)/.exec(className || '')
                  return match ? (
                    <SyntaxHighlighter
                      ref={ref as React.LegacyRef<SyntaxHighlighter> | undefined}
                      {...rest}
                      PreTag="div"
                      children={String(children).replace(/\n$/, '')}
                      language={match[1]}
                      style={solarizedDarkAtom}
                    />
                  ) : (
                    <code ref={ref} {...rest} className={className}>
                      {children}
                    </code>
                  )
                }
              }}
              />
            </div>
          )}
        </Fragment>
      </main>
    </div>
  );
}

export default App;
