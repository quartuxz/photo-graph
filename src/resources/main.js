


async function main(){
  
  await GraphNode.loadNodeTemplates();
  Mat3.test();
  let canvas = document.getElementById("canvas");
  let context = canvas.getContext("2d")
  let graph = new Graph(context);
  let ui = new UI(graph,canvas, context);
}

main();
