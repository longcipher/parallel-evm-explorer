import React, { useEffect, useRef } from 'react';
import { Graph } from 'graphology';
import Sigma from 'sigma';

const GraphComponent = ({ graphData }) => {
  const containerRef = useRef(null);
  const sigmaInstanceRef = useRef(null);

  useEffect(() => {
    if (containerRef.current) {
      // 创建 Graphology 图实例
      const graph = new Graph();
      graphData.nodes.forEach((node) => {
        graph.addNode(node.id, { ...node });
      });
      graphData.edges.forEach((edge) => {
        graph.addEdge(edge.source, edge.target, { ...edge });
      });

      // 初始化 Sigma 实例
      sigmaInstanceRef.current = new Sigma(graph, containerRef.current);
    }

    return () => {
      // 清理 Sigma 实例
      sigmaInstanceRef.current?.kill();
    };
  }, [graphData]);

  useEffect(() => {
    const container = containerRef.current;

    if (container) {
      const resizeObserver = new ResizeObserver(() => {
        // 监听尺寸变化，刷新布局
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
        width: '100%',
        height: '100%',
      }}
    />
  );
};

export default GraphComponent;
