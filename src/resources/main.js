


async function main(){
  
  await GraphNode.loadNodeTemplates();
  Mat3.test();

  let graph = new Graph();
  let ui = new UI(graph,document.getElementById("canvas"));
}

main();
