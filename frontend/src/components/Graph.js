import dynamic from "next/dynamic";
import React, { useEffect, useRef } from "react";

// Dynamically import Sigma, ensuring it only loads on the client side
const GraphComponent = dynamic(
  () =>
    import("sigma").then((Sigma) => {
      return ({ graphData }) => {
        const containerRef = useRef(null);
        const sigmaInstanceRef = useRef(null);

        useEffect(() => {
          if (containerRef.current) {
            const { Graph } = require("graphology"); // Dynamically import graphology on the client side
            const graph = new Graph();

            // Add nodes and edges to the graph
            for (const node of graphData.nodes) {
              graph.addNode(node.id, { ...node });
            }
            for (const edge of graphData.edges) {
              graph.addEdge(edge.source, edge.target, { ...edge });
            }

            // Initialize Sigma instance
            sigmaInstanceRef.current = new Sigma.default(
              graph,
              containerRef.current,
            );
          }

          // Clean up the Sigma instance on component unmount
          return () => {
            sigmaInstanceRef.current?.kill();
          };
        }, [graphData]);

        useEffect(() => {
          const container = containerRef.current;

          if (container) {
            const resizeObserver = new ResizeObserver(() => {
              sigmaInstanceRef.current?.refresh();
            });

            // Observe container resizing to refresh the Sigma instance
            resizeObserver.observe(container);

            // Disconnect the ResizeObserver on cleanup
            return () => {
              resizeObserver.disconnect();
            };
          }
        }, []);

        return (
          <div
            ref={containerRef}
            style={{
              width: "100%",
              height: "100%",
            }}
          />
        );
      };
    }),
  { ssr: false }, // Disable SSR
);

export default GraphComponent;
