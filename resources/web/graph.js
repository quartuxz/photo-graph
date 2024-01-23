let graphFile = getCookie("graphFile");



let graphFontStyle = "15px seif";


class NodeIO{
    isOutput;
    name;
    hasConnection;
    hasDefault;
  }

  class UIElement{
    type;
    node;
    IOSocket;

  }
  
  
  class GraphNode{
    IOs = [];
    nodeName;
    #position = Vec2(0,0);
    #positionCorner = Vec2(100,100);
    #transformation = Mat3.identity();
    #objectTransformation = Mat3.identity();
    #context;
    //as relative positions with respect to the node
    inputCircles = [];
    outputCircles = [];
    defaultValues = [];
    id;
    nodesOccupied = 1;
    isInvisible = false;
    value;
    template;
    selected = false;
    static nodeTemplates;
    
    static #chooseCircleStyle(IOType){
      if(IOType == "int"){
        return "rgb(150,200,150)";
      }else if(IOType == "color"){
        return "rgb(200,200,150)";
      }else if(IOType == "float"){
        return "rgb(150,200,200)";
      }else if(IOType == "string"){
        return "rgb(200,150,150)";
      }else if(IOType== "bitmap"){
        return "rgb(150,150,200)";
      }
    }
  
    constructor(nodeName,position,context){
      this.#context = context;

      this.#context.save();
      this.#context.font =  graphFontStyle;
      


      const template = GraphNode.nodeTemplates.get(nodeName);
      let widths = [100,this.#context.measureText(template.name).width+20];
      this.template = template;
      this.nodesOccupied = template.inputNodes.length+1;
      this.nodeName = template.name;
      this.#objectTransformation = Mat3.translate(position.x,position.y);

      
      for(let i = 0; i<template.inputNodes.length;i++){

        if(template.inputNodes[i].hasConnection){
          widths.push(this.#context.measureText(template.inputNodes[i].name).width+20);
        }

      }

      for(let i = 0; i<template.outputNodes.length;i++){
        if(template.outputNodes[i].hasConnection){
          widths.push(this.#context.measureText(template.outputNodes[i].name).width+20);
        }
      }

      let totalWidth = Math.max(...widths);


      let visibleInputSockets = 0;
      for(let i = 0; i<template.inputNodes.length;i++){
        this.defaultValues.push(template.inputNodes[i].defaultValue);

        if(template.inputNodes[i].hasConnection){
          
          this.inputCircles.push({center:Vec2(0,50*(visibleInputSockets+1)),radius:Vec2(10,50*(visibleInputSockets+1)),style:GraphNode.#chooseCircleStyle(template.inputNodes[i].IOType),socketID:i,name:template.inputNodes[i].name });
          visibleInputSockets += 1;
        }

      }
      let visibleOutputSockets = 0;
      for(let i = 0; i<template.outputNodes.length;i++){
        if(template.outputNodes[i].hasConnection){
          this.outputCircles.push({center:Vec2(totalWidth,50*(visibleOutputSockets+1)+25),radius:Vec2(totalWidth+10,50*(visibleOutputSockets+1)+25),style:GraphNode.#chooseCircleStyle(template.outputNodes[i].IOType),socketID:i, name:template.outputNodes[i].name })
          visibleOutputSockets += 1;
        }
      }
      this.#positionCorner = Vec2(totalWidth,50+50*Math.max(visibleInputSockets,visibleOutputSockets));

      this.#context.restore();
    }

    static async loadNodeTemplates(){
      const options = {
        method: "POST"
      };
      let response = await fetch("/retrieveNodeTemplates", options);
      let nodeTemplates = await response.json();
      GraphNode.nodeTemplates = new Map();
      for(const x of nodeTemplates){
        GraphNode.nodeTemplates.set(x.name,x);
      } 
      console.log(GraphNode.nodeTemplates)
    }

    
  
    #getTransformedCircles(circles, transform){
      let ret = [];
      for (const circle of circles){
        let transformed = structuredClone(circle);
        transformed.center = transform.multiply(circle.center);
        transformed.radius = transform.multiply(circle.radius);
        ret.push(transformed)
      }
      return ret;
    }

    getTransformed(transform){
      let transformed = {position: transform.multiply(this.#position), 
        positionCorner : transform.multiply(this.#positionCorner)}
      
      transformed.inputCircles = this.#getTransformedCircles(this.inputCircles,transform);
      transformed.outputCircles = this.#getTransformedCircles(this.outputCircles,transform);
      return transformed;
    }

    getObjectTransformed(){
      return this.getTransformed(this.#objectTransformation);
    }

    getTotalTransformed(){
      return this.getTransformed(this.#transformation.multiply(this.#objectTransformation));
    }

    //returns the UI element which is pointed to in the position
    getPointed(pos){
      let transformed = this.getTotalTransformed();
      if(!this.isInvisible){
        for (let i = 0; i < transformed.inputCircles.length; i++){
          if(transformed.inputCircles[i].center.distance(pos)<transformed.inputCircles[i].center.distance(transformed.inputCircles[i].radius)){
            let ele = new UIElement();
            ele.type = "input";
            ele.node = this;
            ele.IOSocket = transformed.inputCircles[i].socketID;
            return ele;
          }
        }
        for (let i = 0; i < transformed.outputCircles.length; i++){
          if(transformed.outputCircles[i].center.distance(pos)<transformed.outputCircles[i].center.distance(transformed.outputCircles[i].radius)){
            let ele = new UIElement();
            ele.type = "output";
            ele.node = this;
            ele.IOSocket = transformed.outputCircles[i].socketID;
            return ele;
          }
        }
        if(pos.x >= transformed.position.x && pos.y >= transformed.position.y && pos.x <= transformed.positionCorner.x && pos.y <= transformed.positionCorner.y){
          let ele = new UIElement();
          ele.type = "node";
          ele.node = this;
          return ele;
        }
      }
      return null;
    }
  
    #drawCircles(circles,isInput){
      for (const circle of circles){
        this.#context.save();
        this.#context.beginPath();
        this.#context.arc(circle.center.x, circle.center.y, circle.center.distance(circle.radius) ,0,Math.PI*2,false);
        this.#context.fillStyle= circle.style;
        this.#context.fill();
        this.#context.restore();

        this.#context.save();
        this.#context.fillStyle = "black";

        this.#context.font =  graphFontStyle;
        let xoffset = circle.center.x+circle.center.distance(circle.radius)+3;
        if(!isInput){
          xoffset = circle.center.x-circle.center.distance(circle.radius)-this.#context.measureText(circle.name).width-3;
        }
        this.#context.fillText(circle.name,xoffset,circle.center.y);
        this.#context.restore();

      }
    }
  
    draw(){
      if(!this.isInvisible){
        let transformed = this.getObjectTransformed();
        this.#context.save();
        this.#context.fillStyle = "rgb(60,100,60)";
        this.#context.strokeStyle = "rgb(100,60,60)";
        if(this.selected){
          this.#context.strokeStyle = "rgb(200,120,120)";
          this.#context.lineWidth = 4;
        }
        this.#context.beginPath();
        this.#context.roundRect(transformed.position.x,transformed.position.y,transformed.positionCorner.x-transformed.position.x,transformed.positionCorner.y-transformed.position.y,5);
        this.#context.fill();
        this.#context.stroke();
        this.#context.restore();

        this.#context.save();
        this.#context.fillStyle = "black";
        this.#context.font = graphFontStyle;
        this.#context.fillText(this.nodeName,transformed.position.x+5, transformed.position.y+20);
        this.#context.restore();


        this.#drawCircles(transformed.inputCircles,true);
        this.#drawCircles(transformed.outputCircles,false);
      }
    }
  
    transform(transformation){
      this.#transformation = this.#transformation.multiply(transformation);
    }

    objectTransform(transformation){
      this.#objectTransformation = this.#objectTransformation.multiply(transformation);
    }
  
    set transformation(transformation){
      this.#transformation = transformation;
    }

    set objectTransformation(transformation){
      this.#objectTransformation = transformation;
    }
    
  }
  
  
  class Command{
    name;
    args;
    constructor(name, args){
      this.name = name;
      this.args = args;
    }
  }

  class Line{
    fromID;
    toID;
    fromSocket;
    toSocket;
    constructor(fromID, fromSocket, toID, toSocket){
      this.fromID = fromID;
      this.fromSocket = fromSocket;
      this.toID = toID;
      this.toSocket = toSocket;
    }

    commandForm(){
      return [this.fromID.toString(),this.fromSocket.toString(),this.toID.toString(),this.toSocket.toString()];
    }

    static parseCommand(command){
      return new Line(Number(command.args[0]),Number(command.args[1]),Number(command.args[2]),Number(command.args[3]));
    }
  }

  class Graph{
    #nodes = [];
    #scale = 1;
    #transformation = Mat3.identity();
    #lines = [];
    #currentID = 0;
    #context;
    #commandRequestQueue = [];
  
    constructor(context,graphID){

      this.#context = context;
      this.#_addNode(new GraphNode("Final",Vec2(0,0),this.#context));
    }
    
    getLineByInput(inputID, inputSocket){
      for(const line of this.#lines){
        if(line.toID == inputID && line.toSocket == inputSocket){
          return line;
        }
      }
      return null;
    }

    #_removeLine(line){
      for(let i = 0; i<this.#lines.length;i++){
        if(this.#lines[i].toID == line.toID && this.#lines[i].toSocket == line.toSocket){
          this.#lines.splice(i,1);
        }
      }
    }

    removeLine(line,callback){
      this.#registerCommands([new Command("removeEdge",line.commandForm())],callback);
      this.#_removeLine(line);
    }

    getNode(id){
      for(const node of this.#nodes){
        if(node.id == id){
          return node;
        }
      }
    }
    #_addNode(node){
      node.transformation = this.#transformation;
      node.id = this.#currentID;
      this.#currentID += node.nodesOccupied;
      this.#nodes.push(node);
    }
  
    async addNamedNode(nodeName,position){
      this.#registerCommands([new Command("addNode", [nodeName,position.x.toString(),position.y.toString()])],null);

      this.#_addNode(new GraphNode(nodeName,position,this.#context));
    }

    getNodeIndex(id){
      let toRemovePos = -1;
      for(let i = 0; i< this.#nodes.length;i++){
        if(this.#nodes[i].id == id){
          toRemovePos=i;
        }
      }
      return toRemovePos;
    }

    #_removeNode(id){
      this.#nodes.splice(this.getNodeIndex(id),1);
      let toRemove = [];
      for(const line of this.#lines){
        if(line.toID == id || line.fromID == id){
          toRemove.push(line);
        }
      }
      for(const line of toRemove){
        this.#_removeLine(line);
      }
    }

    removeNode(id,callback){
      this.#registerCommands([new Command("removeNode",[id.toString()])],callback);
      this.#_removeNode(id);
    }

    #_modifyDefault(node, nodeID,parameters){
      //parameters is greater than 1 if we are dealing with a 4-channel color
      if(parameters.length > 1){
        node.defaultValues[nodeID-node.id-1] = parameters;
      }else{
        node.defaultValues[nodeID-node.id-1] = parameters[0];
      }
    }
    //the node whose default value is being modified, the nodeID of the default value and the parameters being changed(could be a number a string or a 4-value array for color)
    modifyDefault(node, nodeID,parameters,callback){
      let args = [nodeID.toString(),node.id.toString()];
      for(const parameter of parameters){
        args.push(parameter.toString());
      }
      this.#registerCommands([new Command("modifyDefault",args)],callback);
      this.#_modifyDefault(node, nodeID,parameters);
    }

    #_addLine(line){
      for(let i = 0; i<this.#lines.length;i++){
        if(this.#lines[i].toID == line.toID && this.#lines[i].toSocket == line.toSocket){
          this.#lines.splice(i,1);
        }
      }
      this.#lines.push(line);
    }

    addLine(line, callback){
      this.#_addLine(line);
      let inner = async () => {
        this.#registerCommands([new Command("addEdge",line.commandForm())],callback,(succesful)=>{ if(!succesful){this.#_removeLine(line);}});
      };
      inner();
      
    }
    //commands are sent to be executed server-side
    #registerCommands(commands, callback1,callback2=null){
        let inner = async (first=false)=> {
          let body = {commands:commands};
          const options = {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(body)
          };
          let response = await fetch("/command", options);
          if(response.status==401){window.location.href = "login";}
          let final = await response.text();
          if(callback2 != null){
            callback2(final=="ok");
          }
          console.log("something");
          if(first){
            this.#commandRequestQueue.shift();
            while(this.#commandRequestQueue.length > 0){
              await this.#commandRequestQueue[0]();
              this.#commandRequestQueue.shift();
            }
            if(callback1 != null){
              await callback1();
            }
          }
          
      }
      if(this.#commandRequestQueue.length == 0){
        this.#commandRequestQueue.push(inner);
        inner(true);
        
      }else{
        this.#commandRequestQueue.push(inner);
      }
    }

    interpretCommands(commands){
      //commands need to be executed client-side
      for(const command of commands){
        switch(command.name){
          
          case "moveNode": this.#nodes[this.getNodeIndex(Number(command.args[0]))].objectTransform(Mat3.translate(Number(command.args[1]),Number(command.args[2]))); break;
          case "addNode": this.#_addNode(new GraphNode(command.args[0],Vec2(Number(command.args[1]),Number(command.args[2])), this.#context)); break;
          case "addEdge": this.#_addLine(Line.parseCommand(command)); break;
          case "removeEdge": this.#_removeLine(Line.parseCommand(command)); break;
          case "removeNode": this.#_removeNode(Number(command.args[0])); break;
          case "modifyDefault": {
              let parameters=[];
              let node = this.#nodes[this.getNodeIndex(Number(command.args[1]))];
              let defaultNodeID = Number(command.args[0]);
              switch(node.template.inputNodes[defaultNodeID-node.id-1].IOType){
                case "int": parameters.push(Number(command.args[2])); break;
                case "float": parameters.push(Number(command.args[2])); break;
                case "string": parameters.push(command.args[2]); break;
                case "color": parameters = [Number(command.args[2]),Number(command.args[3]),Number(command.args[4]),Number(command.args[5])]; break;
              }
              this.#_modifyDefault(node, defaultNodeID,parameters);
              break;
            };
          default: throw new Error("command not recognized!"); break;
        }
      }
    }

    registerNodeMoveCommand(nodeID, delta){
      this.#registerCommands([new Command("moveNode",[nodeID.toString(),delta.x.toString(),delta.y.toString()])]);
    }

    draw(){
  
      for (const line of this.#lines){
        let from = null;
        for(const circle of this.getNode(line.fromID).getObjectTransformed().outputCircles){
          if(circle.socketID == line.fromSocket){
            from = circle.center;
          }
        }

        let to = null;
        for(const circle of this.getNode(line.toID).getObjectTransformed().inputCircles){
          if(circle.socketID == line.toSocket){
            to = circle.center;
          }
        }
        this.#context.save();
        this.#context.beginPath();
        this.#context.moveTo(from.x,from.y);
        this.#context.lineTo(to.x,to.y);
        this.#context.strokeStyle= 'cyan';
        this.#context.lineWidth = 4;
        this.#context.stroke();
        this.#context.restore();
      }

      for (const node of this.#nodes){
        node.draw();
      }

    }
  
    transform(transformation){
      this.#transformation = this.#transformation.multiply(transformation);
      for (const node of this.#nodes){
        node.transformation = this.#transformation;
      }
    }
    
    //returns the UI element which is pointed to in the position for any node
    getPointed(pos){
      for(let i = this.#nodes.length-1; i>=0; i--){
        let node = this.#nodes[i];
        let pointed = node.getPointed(pos);
        if(pointed != null){
          return pointed;
        }
      }
      return null;
    }
  
    getTransformedPos(mousePos){
      return this.#transformation.inverse().multiply(mousePos);
    }
    
    get nodes(){
      return this.#nodes;
    }

    get transformation(){
      return this.#transformation;
    }
  
    set transformation(transformation){
      this.#transformation = transformation;
      for (let node of this.#nodes){
        node.transformation = this.#transformation;
      }
    }
  
    zoom(mousePos, zoom){
  
      this.transform(Mat3.scale(zoom));
      //this.transform(Mat3.translate(mousePos.x, mousePos.y));
  
    }
  }
  
