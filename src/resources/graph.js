



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
    //as relative positions with respect to the node
    inputCircles = [];
    outputCircles = [];
    defaultValues = [];
    id;
    nodesOccupied = 1;
    isInvisible = false;
    value;
    template;
    static nodeTemplates;
    
    
  
    constructor(nodeName,position){

      const template = GraphNode.nodeTemplates.get(nodeName);
      this.template = template;
      this.nodesOccupied = template.inputNodes.length+1;
      this.nodeName = template.name;
      this.#objectTransformation = Mat3.translate(position.x,position.y);
      this.#positionCorner = Vec2(100,50+50*Math.max(template.inputNodes.length,template.outputNodes.length));
      let offset = 1;
      for(let i = 0; i<template.inputNodes.length;i++){
        this.defaultValues.push(template.inputNodes[i].defaultValue);

        if(template.inputNodes[i].hasConnection){

          this.inputCircles.push({center:Vec2(0,50*offset),radius:Vec2(10,50*offset),style:'green',socketID:i });
        }
        offset += 1;

      }
      offset = 1;
      for(let i = 0; i<template.outputNodes.length;i++){
        if(template.outputNodes[i].hasConnection){
          this.outputCircles.push({center:Vec2(100,50*offset),radius:Vec2(100+10,50*offset),style:'green',socketID:i })
        }
        offset += 1;
      }
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
  
    #drawCircles(context, circles){
      for (const circle of circles){
        context.save();
        context.beginPath();
        context.arc(circle.center.x, circle.center.y, circle.center.distance(circle.radius) ,0,Math.PI*2,false);
        context.fillStyle= circle.style;
        context.fill();
        context.restore();
      }
    }
  
    draw(context){
      if(!this.isInvisible){
        let transformed = this.getObjectTransformed();
        context.save();
        context.fillStyle = 'blue';
        context.strokeStyle = 'red';
        context.beginPath();
        context.roundRect(transformed.position.x,transformed.position.y,transformed.positionCorner.x-transformed.position.x,transformed.positionCorner.y-transformed.position.y,5);
        context.fill();
        context.stroke();
        context.restore();

        context.save();
        context.fillStyle = "green";
        context.font = ((transformed.positionCorner.x-transformed.position.x)/5)+"px seif";
        context.fillText(this.nodeName,transformed.position.x+20, transformed.position.y+20);
        context.restore();


        this.#drawCircles(context, transformed.inputCircles);
        this.#drawCircles(context, transformed.outputCircles);
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
  }

  class Graph{
    #nodes = [];
    #scale = 1;
    #transformation = Mat3.identity();
    #lines = [];
    #currentID = 0;
  
    constructor(){
      this.#_addNode(new GraphNode("Final",Vec2(0,0)));
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

    removeLine(line){
      this.#registerCommands([new Command("removeEdge",line.commandForm())]);
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
  
    addNamedNode(nodeName,position){
      this.#registerCommands([new Command("addNode", [nodeName,position.x.toString(),position.y.toString()])]);
      this.#_addNode(new GraphNode(nodeName,position));
    }

    #_modifyDefault(node, nodeID,parameters){
      //parameters is greater than 1 if we are dealing with a 4-channel color
      if(parameters.length > 1){
        console.log(parameters);
        node.defaultValues[nodeID-node.id-1] = parameters;
      }else{
        node.defaultValues[nodeID-node.id-1] = parameters[0];
      }
    }

    modifyDefault(node, nodeID,parameters){
      let args = [nodeID.toString()];
      for(const parameter of parameters){
        args.push(parameter.toString());
      }
      this.#registerCommands([new Command("modifyDefault",args)]);
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

    async addLine(line){
      if(await this.#registerCommands([new Command("addEdge",line.commandForm())])){
        this.#_addLine(line);
      }
    }
    
    async #registerCommands(commands){
        //commands are sent to be executed server-side
        const options = {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(commands)
        };
        let response = await fetch("/command", options);
        let final = await response.text();
        console.log(final);
        if(final != "ok"){

          return false;
        }
        return true;
    }

    #interpretCommands(commands){
      //commands need to be executed client-side
      
    }

    registerNodeMoveCommand(nodeID, delta){
      this.#registerCommands([new Command("moveNode",[nodeID.toString(),delta.x.toString(),delta.y.toString()])]);
    }

    draw(context){
  
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
        context.save();
        context.beginPath();
        context.moveTo(from.x,from.y);
        context.lineTo(to.x,to.y);
        context.strokeStyle= 'cyan';
        context.lineWidth = 4;
        context.stroke();
        context.restore();
      }

      for (const node of this.#nodes){
        node.draw(context);
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
      for (const node of this.#nodes){
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
  
