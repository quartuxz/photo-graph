
function saveOnclick(){

  const options = {
      method: "POST",
      headers: {
          "Content-Type": "application/json",
      },
      body: JSON.stringify({fileName:graphFile})
  };
  fetch("/saveGraph", options);
  return true;
}



async function main(){
  
  await GraphNode.loadNodeTemplates();
  document.getElementById("graphName").innerHTML = graphFile;
  document.getElementById("saveButton").onclick = saveOnclick;
  document.getElementById("uploadImageButton").onclick = ()=>{ window.location.href = domainName+"upload_image.html";};
  document.getElementById("selectGraphButton").onclick = ()=>{window.location.href = domainName+"graph_selector.html";};
  document.getElementById("loginPage").href = domainName+"login.html";


  let canvas = document.getElementById("canvas");
  let context = canvas.getContext("2d")
  let graph = new Graph(context);

  const options = {
    method: "POST",
  };
  let response = await fetch("/retrieveGraph", options);
  if(response.status==401){window.location.href = domainName+"login.html";}
  let commandHistory = await response.json();
  
  graph.interpretCommands(commandHistory);

  let ui = new UI(graph,canvas, context);
  document.getElementById("centerGraphButton").onclick = ()=>{ui.resetView(); };
}

main();
