// oxlint-disable no-unused-vars
"use client";

import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

export function MarkdownRenderer({
  content,
  className = "",
}: MarkdownRendererProps) {
  return (
    <div className={`prose prose-sm max-w-none ${className}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          // Headings
          h1: ({ node, ...props }) => (
            <h1 className="text-lg font-bold mt-4 mb-2" {...props} />
          ),
          h2: ({ node, ...props }) => (
            <h2 className="text-base font-bold mt-3 mb-2" {...props} />
          ),
          h3: ({ node, ...props }) => (
            <h3 className="text-sm font-bold mt-2 mb-1" {...props} />
          ),
          // Paragraphs
          p: ({ node, ...props }) => (
            <p className="mb-2 last:mb-0" {...props} />
          ),
          // Lists
          ul: ({ node, ...props }) => (
            <ul className="list-disc list-inside mb-2 space-y-1" {...props} />
          ),
          ol: ({ node, ...props }) => (
            <ol
              className="list-decimal list-inside mb-2 space-y-1"
              {...props}
            />
          ),
          li: ({ node, ...props }) => <li className="text-sm" {...props} />,
          // Code
          code: ({ node, className, children, ...props }) => {
            const isInline = !className;
            return isInline ? (
              <code
                className="bg-muted px-1 py-0.5 rounded text-xs font-mono"
                {...props}
              >
                {children}
              </code>
            ) : (
              <code
                className="block bg-muted p-2 rounded text-xs font-mono overflow-x-auto"
                {...props}
              >
                {children}
              </code>
            );
          },
          // Links
          a: ({ node, ...props }) => (
            <a
              className="text-primary hover:underline"
              target="_blank"
              rel="noopener noreferrer"
              {...props}
            />
          ),
          // Blockquotes
          blockquote: ({ node, ...props }) => (
            <blockquote
              className="border-l-4 border-muted pl-3 italic my-2"
              {...props}
            />
          ),
          // Tables (GFM)
          table: ({ node, ...props }) => (
            <table
              className="border-collapse border border-muted w-full my-2 text-sm"
              {...props}
            />
          ),
          thead: ({ node, ...props }) => (
            <thead className="bg-muted" {...props} />
          ),
          th: ({ node, ...props }) => (
            <th
              className="border border-muted px-2 py-1 font-semibold"
              {...props}
            />
          ),
          td: ({ node, ...props }) => (
            <td className="border border-muted px-2 py-1" {...props} />
          ),
          // Horizontal rule
          hr: ({ node, ...props }) => (
            <hr className="my-3 border-muted" {...props} />
          ),
          // Emphasis
          strong: ({ node, ...props }) => (
            <strong className="font-bold" {...props} />
          ),
          em: ({ node, ...props }) => <em className="italic" {...props} />,
          // Strikethrough (GFM)
          del: ({ node, ...props }) => (
            <del className="line-through" {...props} />
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
