
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
  selected;
  position;
  ui;
  nodeIDs = new Map();
  
  constructor(type, parameter, ui){
    this.type = type;
    this.ui = ui;
    if(type=="default"){
      document.getElementById("contextInner").innerHTML = "";
    }
    else if(type == "manipulate"){
      document.getElementById("contextInner").innerHTML = "";
      let contents = "Edit "+ parameter.nodeName +": <br> <form id=\"manipulateForm\" >";
      console.log(parameter.template);
      let isEditable = false;
      for(let i = 0; i < parameter.template.inputNodes.length;i++){

        const inode = parameter.template.inputNodes[i];
        this.nodeIDs.set(inode.name,parameter.id+i+1);
        if(inode.canAlterDefault){
          isEditable = true;
          contents += "<label for=\""+ inode.name +"\">"+inode.name+"</label> <br>";
          let value = parameter.defaultValues[i];
          if(inode.IOType == "color"){
            contents += "<input type=\"color\" id=\""+inode.name+"\" name=\""+inode.name+"\" value=\""+RGBToHexadecimal(value[0],value[1],value[2])+"\"></input> <br>";
            contents += "<label for=\"alpha\">alpha:</label> <br>";
            contents += "<input type=\"text\" id=\"alpha\" name=\"alpha\" value=\""+value[3]+"\"></input>";
          }else{
            contents += "<input type=\"text\" id=\""+inode.name+"\" name=\""+inode.name+"\" value=\""+value+"\"></input>";
          }

        }
      }
      if(!isEditable){
        return;
      }
      console.log(contents);
      contents += "<input type=\"submit\" value=\"Change\"></input>  </form>";
      document.getElementById("contextInner").innerHTML = contents;
      document.getElementById("manipulateForm").onsubmit = this.onSubmit.bind(this);
      
      this.selected = parameter;

    }else if(type=="create"){
      document.getElementById("contextInner").innerHTML = "create node: <br> here";
      this.position=parameter;
    }
  }

  onSubmit(){
    for(const inode of this.selected.template.inputNodes){
      if(inode.IOType == "color"){
        let color = hexadecimalToRGB(document.getElementById(inode.name).value);
        color.push(parseInt(document.getElementById("alpha").value));
        this.ui.graph.modifyDefault(this.selected,this.nodeIDs.get(inode.name),color);
      }else{
        this.ui.graph.modifyDefault(this.selected,this.nodeIDs.get(inode.name),[document.getElementById(inode.name).value]);
      }

    }
    this.ui.process();
    return false;
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
  
    constructor(graph, canvas){
      this.contextMenu = new ContextMenu("default",null,this);

      this.graph = graph;
      this.canvas = canvas;
      this.context = canvas.getContext("2d");
  
      this.canvas.height = 600;
      this.canvas.width = 600;
  
      this.canvas.addEventListener('mousedown',this.mouseDown.bind(this), false);
      this.canvas.addEventListener('mouseup',this.mouseUp.bind(this), false);
      this.canvas.addEventListener('mousemove',this.mouseMove.bind(this), false);
      this.canvas.addEventListener('wheel',this.wheel.bind(this), false);
      this.canvas.addEventListener('dblclick',this.dblClick.bind(this), false);
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
  
  
    async process(){
      let data;
      let response = fetch("/process",{method:"POST"}).then(response=>{response.blob().then(blobResponse => {this.background.src=window.URL.createObjectURL(blobResponse);});});
    }
  
    draw(){
      this.context.save();
      this.context.setTransform(1,0,0,1,0,0);
      this.context.clearRect(0,0,this.canvas.width,this.canvas.height);
      //draw background image
      this.context.drawImage(this.background,0,0,this.canvas.width,this.canvas.height);
      this.context.restore();
  
  
      this.graph.draw(this.context);
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
            this.contextMenu = new ContextMenu("manipulate",this.selecting.node,this);
          }

          if(this.selecting.type == "input"){
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
  
    dblClick(evt){
      let mousePos = this.#getMousePos(evt);
      //let transform = context.getTransform().invertSelf();
      let transformed = this.graph.getTransformedPos(mousePos);
      //let transformed = Vec2(mousePos.x*transform.a+mousePos.y*transform.c + transform.e, mousePos.x*transform.b+mousePos.y*transform.d+transform.f);
      transformed.z = 1;
      this.contextMenu = new ContextMenu("create",transformed,this);
      this.graph.addNamedNode("Color to image", transformed);
      this.draw();
    }
  
  }
  
  
  
  
  