




async function main(){
  
  await GraphNode.loadNodeTemplates();
  document.getElementById("graphName").innerHTML = graphName;
  document.getElementById("uploadImageButton").onclick = ()=>{ window.location.href = "upload_image.html";};
  document.getElementById("selectGraphButton").onclick = ()=>{window.location.href = "graph_selector.html";};
  document.getElementById("loginPage").href = "login.html";


  let canvas = document.getElementById("canvas");
  let context = canvas.getContext("2d")
  let graph = new Graph(context);

  const options = {
    method: "POST",
    body:graphName
  };
  let response = await fetch("/retrieveGraph", options);
  if(response.status==401){window.location.href = "login.html";}
  let commandHistory = await response.json();
  
  graph.interpretCommands(commandHistory);

  let ui = new UI(graph,canvas, context);
  document.getElementById("centerGraphButton").onclick = ()=>{ui.resetView(); };
}

main();
