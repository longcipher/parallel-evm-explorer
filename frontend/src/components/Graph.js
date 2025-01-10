import { Graph } from "graphology";
import React, { useEffect, useRef } from "react";
import Sigma from "sigma";

const GraphComponent = ({ graphData }) => {
  const containerRef = useRef(null);
  const sigmaInstanceRef = useRef(null);

  useEffect(() => {
    if (containerRef.current) {
      const graph = new Graph();
      for (const node of graphData.nodes) {
        graph.addNode(node.id, { ...node });
      }
      for (const edge of graphData.edges) {
        graph.addEdge(edge.source, edge.target, { ...edge });
      }
      sigmaInstanceRef.current = new Sigma(graph, containerRef.current);
    }

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

      resizeObserver.observe(container);

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

export default GraphComponent;
