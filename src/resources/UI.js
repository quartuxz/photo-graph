
function hexadecimalToRGB(hex){
  let rgb = [];
  for(let i = 0; i< 3; i++){
    rgb.push(parseInt(hex[i*2+1]+hex[i*2+2],16));
  }
  return rgb;
}

function RGBToHexadecimal(r,g,b){
  hex = "#";
  let r0x = r.toString(16);
  let g0x = g.toString(16);
  let b0x = b.toString(16);

  if(r0x.length < 2){
    r0x = "0"+r0x;
  }
  if(g0x.length < 2){
    g0x = "0"+g0x;
  }
  if(b0x.length < 2){
    b0x = "0"+b0x;
  }
  hex += r0x+g0x+b0x;
  return hex;
}

class ContextMenu{
  type;
  selected = null;
  position;
  ui;
  nodeProperties = new Map();
  
  constructor(type, parameter, ui){
    this.type = type;
    this.ui = ui;
    if(type=="default"){
      document.getElementById("contextInner").innerHTML = "";
    }
    else if(type == "manipulate"){
      this.selected = parameter;
      this.selected.selected = true;
      document.getElementById("contextInner").innerHTML = "";
      let contents = "Edit \""+ parameter.nodeName +"\": <br> <form id=\"manipulateForm\" >";
      console.log(parameter.template);
      let isEditable = false;
      for(let i = 0; i < parameter.template.inputNodes.length;i++){

        const inode = parameter.template.inputNodes[i];
        this.nodeProperties.set(inode.name,{id:parameter.id+i+1,hasConnection:false});
        if(inode.canAlterDefault){
          isEditable = true;

          let value = parameter.defaultValues[i];
          if(this.ui.graph.getLineByInput(parameter.id,i)!= null){
            contents += inode.name+"<br> connected<br>";
            this.nodeProperties.get(inode.name).hasConnection = true;
          }
          else if(inode.IOType == "color"){
            contents += "<label for=\""+ inode.name +"\">"+inode.name+"</label> <br>";
            contents += "<input type=\"color\" id=\""+inode.name+"\" name=\""+inode.name+"\" value=\""+RGBToHexadecimal(value[0],value[1],value[2])+"\"></input> <br>";
            contents += "<label for=\"alpha\">alpha:</label> <br>";
            contents += "<input type=\"text\" id=\"alpha\" name=\"alpha\" value=\""+value[3]+"\"></input> <br>";
          }else if(inode.presetValues != null){
            contents += inode.name+":<br>";
            for(let o = 0; o < inode.presetValues.length;o++){
              let checked = "";
              if(o == value){
                console.log(o);
                checked = "checked";
              }

              contents += "<input type=\"radio\" id=\""+inode.presetValues[o]+"\" name=\"preset\" value\""+o+"\" "+checked+"></input>";
              contents += "<label for=\""+inode.presetValues[o]+"\">"+inode.presetValues[o]+"</label><br>";

            }
          }else{
            contents += "<label for=\""+ inode.name +"\">"+inode.name+"</label> <br>";
            contents += "<input type=\"text\" id=\""+inode.name+"\" name=\""+inode.name+"\" value=\""+value+"\"></input> <br>";
          }

        }
      }
      if(!isEditable){
        return;
      }
      contents += "<input type=\"submit\" value=\"Change\"></input>  </form>";
      document.getElementById("contextInner").innerHTML = contents;
      document.getElementById("manipulateForm").onsubmit = this.onSubmitManipulate.bind(this);


    }else if(type=="create"){
      let contents = "create node: <br> <form id=\"createForm\">";

      for(const [key,template] of GraphNode.nodeTemplates){

        contents += "<input type=\"radio\" id=\""+template.name+"\" name=\"create\" value=\""+template.name+"\"></input>";
        contents += "<label for=\""+ template.name+"\">"+template.name +"</label><br>";
      }

      contents += "<input type=\"submit\" value=\"Create\"></input> </form>";
      document.getElementById("contextInner").innerHTML = contents;
      document.getElementById("createForm").onsubmit = this.onSubmitCreate.bind(this);
      this.position=parameter;
    }
  }

  onSubmitManipulate(){
    for(const inode of this.selected.template.inputNodes){
      if(this.nodeProperties.get(inode.name).hasConnection){
        //nothing to do
      }
      else if(inode.IOType == "color"){
        let color = hexadecimalToRGB(document.getElementById(inode.name).value);
        color.push(parseInt(document.getElementById("alpha").value));
        this.ui.graph.modifyDefault(this.selected,this.nodeProperties.get(inode.name).id,color);
      }else if(inode.presetValues != null){
        for(let i = 0; i < inode.presetValues.length; i++){
          if(document.getElementById(inode.presetValues[i]).checked){
            this.ui.graph.modifyDefault(this.selected,this.nodeProperties.get(inode.name).id,[i]);
          }

        }
      }else{
        this.ui.graph.modifyDefault(this.selected,this.nodeProperties.get(inode.name).id,[document.getElementById(inode.name).value]);
      }

    }
    this.ui.process();
    return false;
  }

  onSubmitCreate(){
      for(const [key,template] of GraphNode.nodeTemplates){
        if(document.getElementById(template.name).checked){
          this.ui.graph.addNamedNode(template.name, this.position);
        }
      }
      this.ui.draw();
      this.position.x += 10;
      this.position.y += 10;
      return false;
  }

  draw(){
    if(this.type == "create"){
      let rectWidth = 5;
      let rectHeight = 20;
      let context = this.ui.context;
      context.fillStyle = "rgb(100,150,150)";
      context.fillRect(this.position.x-rectWidth/this.ui.scale/2,this.position.y-rectHeight/this.ui.scale/2,rectWidth/this.ui.scale,rectHeight/this.ui.scale);
      context.fillRect(this.position.x-rectHeight/this.ui.scale/2,this.position.y-rectWidth/this.ui.scale/2,rectHeight/this.ui.scale,rectWidth/this.ui.scale);
    }
  }
}


class UI{
    origin = Vec2(0,0);
    scale = 1;
    previousMousePosition = Vec2(0,0);
    isLeftMouseDown = false;
    isRightMouseDown = false;
    isMiddleMouseDown = false;
    selecting = null;
    nodeMoveDelta = Vec2(0,0);
    graph;
    canvas;
    context;
    contextMenu;
    background = new Image();
  
    constructor(graph, canvas,context){
      this.contextMenu = new ContextMenu("default",null,this);

      this.graph = graph;
      this.canvas = canvas;
      this.context = context;
  
      this.canvas.height = 600;
      this.canvas.width = 1000;
  
      this.canvas.addEventListener('mousedown',this.mouseDown.bind(this), false);
      this.canvas.addEventListener('mouseup',this.mouseUp.bind(this), false);
      this.canvas.addEventListener('mousemove',this.mouseMove.bind(this), false);
      this.canvas.addEventListener('wheel',this.wheel.bind(this), false);
      this.canvas.addEventListener('dblclick',this.dblClick.bind(this), false);
      this.canvas.addEventListener('keydown',this.keydown.bind(this), false);
      this.process();
      let draw= this.draw.bind(this);
      this.background.onload = ()=>{
        this.draw();
      }
      this.draw();
    }
  
    #translate(dx,dy){
        this.context.translate(dx,dy);
  
        this.graph.transform(Mat3.translate(dx,dy));
    }
  
    #scale(zoom){
      this.context.scale(zoom,zoom);
      this.graph.transform(Mat3.scale(zoom));
      this.scale *= zoom;
    }
  
    #getMousePos(evt){
      let rect = this.canvas.getBoundingClientRect();
      return Vec2(evt.clientX - rect.left, evt.clientY - rect.top);
    }
  
    #changeContextMenu(type, parameter){

      if(this.contextMenu.selected != null){
        this.contextMenu.selected.selected = false;
      }
      

      this.contextMenu = new ContextMenu(type,parameter,this);
    }

    async process(){
      const options = {
        method: "POST",
        headers: {
          "Content-Type": "text",
        },
        body: graphID.toString()
      };
      fetch("/process",options).then(response=>{ if(response.status==401){window.location.href = domainName+"login";} response.blob().then(blobResponse => {this.background.src=window.URL.createObjectURL(blobResponse);});});
    }

    #drawTransparencyBackground(){
      
      let xDim = this.canvas.width/30;
      let yDim = this.canvas.height/30;
      for(let i = 0; i<xDim+1; i++){
        for(let o = 0; o<yDim+1; o++){
          if((i %2==0 && o %2!=0)||(i %2!=0 && o %2==0)){
            this.context.fillStyle = "rgb(125,125,125)";
          }else{
            this.context.fillStyle = "rgb(200,200,200)";
          }
          this.context.fillRect(i*30,o*30,30,30);
        }
      }
    }
  
    draw(){
      this.context.save();
      this.context.setTransform(1,0,0,1,0,0);
      this.context.clearRect(0,0,this.canvas.width,this.canvas.height);
      this.#drawTransparencyBackground();
      //draw background image
      if(this.background.width > this.background.height){
        this.context.drawImage(this.background,0,(this.canvas.height-this.background.height*(this.canvas.width/this.background.width))/2,this.canvas.width,this.background.height*(this.canvas.width/this.background.width));
      }else{
        this.context.drawImage(this.background,(this.canvas.width-this.background.width*(this.canvas.height/this.background.height))/2,0,this.background.width*(this.canvas.height/this.background.height),this.canvas.height);
      }

      this.context.restore();
  
  
      this.graph.draw(this.context);
      this.contextMenu.draw();
    }
  
    async mouseDown(evt){
      if(evt.button == 0){
        if (evt.detail > 1) {
          evt.preventDefault();
        }
        this.isLeftMouseDown =true;
        this.selecting = this.graph.getPointed(this.#getMousePos(evt));

        if(this.selecting != null){

          if(this.selecting.type == "node"){
            
            this.#changeContextMenu("manipulate",this.selecting.node);
          }

          if(this.selecting.type == "input"){
            this.#changeContextMenu("default",null);
            let manipulatedLine = this.graph.getLineByInput(this.selecting.node.id,this.selecting.IOSocket);
            if(manipulatedLine != null){
              this.graph.removeLine(manipulatedLine);
              this.draw();
              this.process();
              this.selecting = new UIElement();
              this.selecting.type = "output";
              this.selecting.node = this.graph.getNode(manipulatedLine.fromID);
              this.selecting.IOSocket = manipulatedLine.fromSocket;
            }
    
            
          }
        
        }
      }else if(evt.button==1){
        this.isMiddleMouseDown = true;
      }else if(evt.button == 2){
        this.isRightMouseDown = true;
  
      }
      
      
    }
  
    async mouseUp(evt){
      //this.selecting = null;
      if(evt.button == 0){
        this.isLeftMouseDown =false;
        if(this.selecting != null){
          let type1 = "output";
          let type2 = "input";
          for(let i = 0; i<2; i++){
            if(this.selecting.type == type1){
              let pointed = this.graph.getPointed(this.#getMousePos(evt));
              if(pointed == null){
                break;
              }
              if(pointed.type == type2 && pointed.node.id != this.selecting.node.id){
                if(type1 == "output"){
                  await this.graph.addLine(new Line(this.selecting.node.id,this.selecting.IOSocket,pointed.node.id,pointed.IOSocket));
                }else{
                  await this.graph.addLine(new Line(pointed.node.id,pointed.IOSocket,this.selecting.node.id,this.selecting.IOSocket));
                }
                this.process();
              }
  
  
            }
            type1 = "input";
            type2 = "output";
          }
          if(this.selecting.type == "node"){
            if(!(this.nodeMoveDelta.x == 0 && this.nodeMoveDelta.y == 0)){
              this.graph.registerNodeMoveCommand(this.selecting.node.id, this.nodeMoveDelta);
              this.nodeMoveDelta = Vec2(0,0);
            }
  
          }
        }
        this.draw();
  
      }else if(evt.button==1){
        this.isMiddleMouseDown = false;
      }else if(evt.button == 2){
        this.isRightMouseDown = false;
      }
    }
  
    mouseMove(evt){
      let mousePos = this.#getMousePos(evt);
      if (this.isRightMouseDown){
          
          let dx = (mousePos.x - this.previousMousePosition.x)/this.scale;
          let dy = (mousePos.y - this.previousMousePosition.y)/this.scale;
          this.origin.x -= dx;
          this.origin.y -= dy;
          this.#translate(dx,dy);
          
          this.draw();
      }
      if (this.isLeftMouseDown && !this.isRightMouseDown){
        if(this.selecting != null){        
          if(this.selecting.type == "node"){
            let movedX = this.graph.getTransformedPos(mousePos).x-this.graph.getTransformedPos(this.previousMousePosition).x;
            let movedY = this.graph.getTransformedPos(mousePos).y-this.graph.getTransformedPos(this.previousMousePosition).y;
            this.nodeMoveDelta.x += movedX;
            this.nodeMoveDelta.y += movedY;
            this.selecting.node.objectTransform(Mat3.translate(movedX,movedY));
          }
  
          this.draw();
          if(this.selecting.type == "output" || this.selecting.type == "input"){
            let from = null;
            if(this.selecting.type == "output"){
              from = this.selecting.node.getObjectTransformed().outputCircles[this.selecting.IOSocket].center;
            }else{
              from = this.selecting.node.getObjectTransformed().inputCircles[this.selecting.IOSocket].center;
            }
            let to = this.graph.getTransformedPos(mousePos);
            this.context.save();
            this.context.beginPath();
            this.context.moveTo(from.x,from.y);
            this.context.lineTo(to.x,to.y);
            this.context.strokeStyle= 'cyan';
            this.context.lineWidth = 4;
            this.context.stroke();
            this.context.restore();
  
          }
        }
      }
  
      this.previousMousePosition = mousePos;
    }
  
    wheel(evt){
      evt.preventDefault();
      // Normalize mouse wheel movement to +1 or -1 to avoid unusual jumps.
      const wheel = evt.deltaY < 0 ? 1 : -1;
      

      if(!((this.scale > 10 && wheel == 1)||(this.scale<0.1 && wheel == -1))){
        let mousePos = this.#getMousePos(evt);
  
        const zoom = Math.exp(wheel * 0.2);

        this.#translate(this.origin.x,this.origin.y);
    
        // Compute the new visible origin. Originally the mouse is at a
        // distance mouse/scale from the corner, we want the point under
        // the mouse to remain in the same place after the zoom, but this
        // is at mouse/new_scale away from the corner. Therefore we need to
        // shift the origin (coordinates of the corner) to account for this.
        //let transformed = graph.getTransformedMouse(mousePos);
        this.origin.x -= mousePos.x/(this.scale*zoom) - mousePos.x/this.scale;
        this.origin.y -= mousePos.y/(this.scale*zoom) - mousePos.y/this.scale;
    
        this.#scale(zoom);
        this.#translate(-this.origin.x,-this.origin.y);
    
    
    
        this.draw();
      }
    }
  
    dblClick(evt){
      let mousePos = this.#getMousePos(evt);
      //let transform = context.getTransform().invertSelf();
      let transformed = this.graph.getTransformedPos(mousePos);
      //let transformed = Vec2(mousePos.x*transform.a+mousePos.y*transform.c + transform.e, mousePos.x*transform.b+mousePos.y*transform.d+transform.f);
      transformed.z = 1;
      this.#changeContextMenu("create",transformed);
      this.draw();
    }

    keydown(evt){
      let code = evt.keyCode;
      switch(code){
        case 88: if(this.selecting != null && this.selecting.node.id != 0){this.#changeContextMenu("default",null); this.graph.removeNode(this.selecting.node.id); this.process()};
        default: ;
      }
    }
  
  }
  
  
  
  
  