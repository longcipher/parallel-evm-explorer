"use client";
import React, { useEffect, useState, useRef } from "react";
import { get } from "@/utils/api";
import Graph from "@/components/Graph";

// 方形蚊香环

export default function Home() {
  const [graphData, setGraphData] = useState({
    // 数据格式
    // nodes: [
    //   { id: 'n1', label: 'Node 1', size: 10, color: '#f00' },
    // ],
    // edges: [
    //   { id: 'e1', source: 'n1', target: 'n2', color: '#ccc', type: 'arrow' },
    // ],
    nodes: [],
    edges: [],
  });
  const [loading, setLoading] = useState(false);
  const [blockNumber, setBlockNumber] = useState(2952107);

  useEffect(() => {
    const url = {
      transactionDag: "/data/evm/transaction-dag",
      analyzerState: "/data/evm/parallel-analyzer-state",
    };

    // 加载交易图数据
    function loadTransactionDag(block_number) {
      setLoading(() => true);
      get(url.transactionDag, { block_number })
        .then((res) => {
          let r = 5;
          let preX;

          // 处理节点数据
          const nodes = (res.transactions || []).map((item, i) => {
            const x =
              i === 0 ? 0 : Math.ceil(i / 4) * ((i - 1) % 4 <= 1 ? 1 : -1) * r;
            const y = i === 0 ? 0 : -preX;
            preX = x;

            return {
              ...item,
              id: item.index,
              label: "" + item.index,
              size: 10,
              x,
              y,
            };
          });

          // 处理边数据
          const edges = (res.dags || []).map((item, i) => ({
            ...item,
            id: "e" + i,
            type: "arrow", // 设置为箭头类型
          }));

          // 更新 graphData 的状态
          setGraphData({ nodes, edges });
        })
        .finally(() => {
          setLoading(() => false);
        });
    }

    // 调用接口并加载数据
    loadTransactionDag(blockNumber);
  }, []); // 仅在组件挂载时调用一次

  return (
    <div className="flex flex-col h-screen gap-4 p-4">
      <nav className="flex gap-4">
        <a className="rounded transition-colors flex items-center justify-center bg-cpurple gap-2 hover:bg-[#AB47BC] dark:hover:bg-[#ccc] h-10 px-8 py-4">
          <i className="iconfont icon-pause"></i>
          {/* <i className="iconfont icon-playfill"></i> */}
        </a>
        <div className="flex gap-1">
          <a className="rounded transition-colors flex items-center justify-center bg-cblue gap-2 hover:bg-[#42A5F5] dark:hover:bg-[#ccc] h-10 px-8 py-4">
            <i className="iconfont icon-zuo"></i>
          </a>
          <a className="rounded transition-colors flex items-center justify-center bg-cblue gap-2 hover:bg-[#42A5F5] dark:hover:bg-[#ccc] h-10 px-8 py-4">
            <i className="iconfont icon-you"></i>
          </a>
        </div>
      </nav>
      <main className="flex-1 flex flex-col rounded border-2 border-orange-500">
        <div className="bg-blue-100 font-600 px-4 py-2 text-lg">
          Block: { blockNumber }
        </div>
        <div className="flex-1">
          {loading && <div className="loading text-center mt-[20%] text-[20px]" >加载中...</div>}
          {/* 在这里渲染图表，传入 graphData */}
          <Graph graphData={graphData} />
        </div>
      </main>
      <footer className="row-start-3 flex gap-6 flex-wrap items-center justify-between"></footer>
    </div>
  );
}
