class Mat3{
  data;

  constructor(data){
    this.data = data;
  }

  clone(){
    var newData = [];

    for (var i = 0; i < self.data.length; i++){
      newData[i] = self.data[i].slice();
    }
    return new Mat3(newData);
  }
  //in case of matrix matrix multiplication multiplies FACTOR*SELF
  multiply(factor){
    if(factor instanceof Mat3){
      let result = Mat3.identity();
      let a0 = factor.data[0][0],
        a1 = factor.data[1][0],
        a2 = factor.data[0][1],
        a3 = factor.data[1][1],
        a4 =factor.data[0][2],
        a5 = factor.data[1][2];
      let b0 = this.data[0][0],
        b1 = this.data[1][0],
        b2 = this.data[0][1],
        b3 = this.data[1][1],
        b4 = this.data[0][2],
        b5 = this.data[1][2];
      result.data[0][0] = a0 * b0 + a2 * b1;
      result.data[1][0] = a1 * b0 + a3 * b1;
      result.data[0][1] = a0 * b2 + a2 * b3;
      result.data[1][1] = a1 * b2 + a3 * b3;
      result.data[0][2] = a0 * b4 + a2 * b5 + a4;
      result.data[1][2] = a1 * b4 + a3 * b5 + a5;
      return result;
      for(let i = 0;i<3;i++){
        for(let j = 0; j <3;j++){
          let oneResult = 0;
          for(let o = 0; o<3;o++){
            oneResult += this.data[i][o]*factor.data[o][j];
          }
          result.data[i][j] = oneResult;
        }
      }
      return result;
    }else if(factor instanceof Vec3){
      return new Vec3(factor.x*this.data[0][0]+factor.y*this.data[0][1]+factor.z*this.data[0][2],
        factor.x*this.data[1][0]+factor.y*this.data[1][1]+factor.z*this.data[1][2],
        factor.x*this.data[2][0]+factor.y*this.data[2][1]+factor.z*this.data[2][2]);
        
    }else if(typeof factor == "number"){
      let result = Mat3.zeros();
      for(let i = 0;i<3;i++){
        for(let j = 0; j <3;j++){
          result.data[i][j] = this.data[i][j]*factor;
        }
      }
      return result;
    }

  }

  determinant(){
    return this.data[0][0]*this.data[1][1]*this.data[2][2] +
    this.data[0][1]*this.data[1][2]*this.data[2][0] +
    this.data[0][2]*this.data[1][0]*this.data[2][1] -
    this.data[0][0]*this.data[1][2]*this.data[2][1] -
    this.data[0][1]*this.data[1][0]*this.data[2][2] -
    this.data[0][2]*this.data[1][1]*this.data[2][0];
  }

  transpose(){
    let result = Mat3.zeros();
    for(let i = 0;i<3;i++){
      for(let j = 0; j <3;j++){
        result.data[j][i] = this.data[i][j];
      }
    }
    return result;
  }

  adjoint(){
    let result = Mat3.zeros();
    let sign = 1;
    for(let i = 0;i<3;i++){
      for(let j = 0; j <3;j++){
        let oneResult = 0;
        let cofactor = [];
        for(let o = 0; o<3;o++){
          for(let p = 0; p<3;p++){
            if(o == i || p == j){
              continue;
            }
            cofactor.push(this.data[o][p]);
          }
        }
        result.data[i][j] = sign*(cofactor[0]*cofactor[3]-cofactor[1]*cofactor[2]);
        sign *= -1;
      }
    }
    return result.transpose();
  }

  inverse(){
    return this.adjoint().multiply(1/this.determinant());
  }

  equals(other){
    for(let i = 0;i<3;i++){
      for(let j = 0; j <3;j++){
        if(this.data[i][j] != other.data[i][j]){
          return false;
        }
      }
    }
    return true;
  }

  static test(){
    let testString = "Mat3 ";
    let test1 = new Mat3([[1,2,-1],[2,1,2],[-1,2,1]]);
    if(test1.determinant() != -16){
      console.log(testString+"test1 failed!");
      console.log("expected : -16 got : "+test1.determinant());
    }
    let test2 = new Mat3([[3/16,1/4,-5/16],[1/4,0,1/4],[-5/16,1/4,3/16]]);
    if(!test1.inverse().equals(test2)){
      console.log(testString+"test2 failed!");
      console.log("expected : "+test2.data+" got : "+test1.inverse().data);
    }
    let test3 = new Mat3([[-3,-4,5],[-4,0,-4],[5,-4,-3]]);
    if(!test1.adjoint().equals(test3)){
      console.log(testString+"test3 failed!");
      console.log("expected : "+test3.data+" got : "+test1.adjoint().data);
    }
    let test41 = new Mat3([[20,10,5],[2,3,4],[6,1,2]]);
    let test42 = new Mat3([[7,3,8],[3,6,2],[4,1,1]]);
    let test4Expected = new Mat3([[190,125,185],[39,28,26],[53,26,52]]);
    let test4Res = test41.multiply(test42);
    if(!test4Res.equals(test4Expected)){
      console.log(testString+"test4 failed!");
      console.log("expected : "+test4Expected.data+" got : "+test4Res.data);
    }

    let test51 = new Mat3([[1,2,3],[4,5,6],[7,8,9]]);
    let test52 = Vec2(10,20);
    let test5Expected = new Vec3(53,146,239);
    let test5Res = test51.multiply(test52);
    if(!test5Res.equals(test5Expected)){
      console.log(testString+"test4 failed!");
      console.log("expected : "+test5Expected.toString()+" got : "+test5Res.toString());
    }
  }

  static zeros(){
    return new Mat3([[0,0,0],[0,0,0],[0,0,0]])
  }

  static identity(){
    return new Mat3([[1,0,0],[0,1,0],[0,0,1]])
  }

  static translate(x,y){
    return new Mat3([[1,0,x],[0,1,y],[0,0,1]])
  }
  static scale(fac){
    return new Mat3([[fac,0,0],[0,fac,0],[0,0,1]])
  }
}

class Vec3{
  x = 0;
  y = 0;
  z = 1;

  constructor(x,y,z){
    this.x = x;
    this.y = y;
    this.z = z;
  }

  add(sum){
    if(sum instanceof Vec3){
      return new Vec3(this.x+sum.x,this.y+sum.y, this.z+sum.z);
    }
  }

  equals(other){
    if(this.x != other.x || this.y != other.y || this.z != other.z){
      return false;
    }
    return true;
  }

  toString(){
    return "x: "+this.x+",y: "+this.y+",z: "+this.z;
  }
}

function Vec2(x,y){
  return new Vec3(x,y,1);
}


class NodeIO{
  isOutput;
  name;
  hasConnection;
  hasDefault;
}


class Node{
  IOs = [];
  nodeName;
  #position = Vec2(0,0);
  #positionCorner = Vec2(100,100);
  #transformation = Mat3.identity();
  //as relative positions with respect to the node
  #inputCircles = [];
  #outputCircles = [];



  constructor(IOs, nodeName, position){
    this.#position = position;
    this.#positionCorner = position.add(new Vec3(100,100,0));
    this.IOs = IOs;
    this.nodeName = nodeName;
    this.#outputCircles.push({center:new Vec2(position.x+100,position.y+50),radius:new Vec2(position.x+100+15,position.y+50),style:'green'});
  }

  #getTransformedCircles(circles, transformed){
    let ret = [];
    for (const circle of circles){
      ret.push({center:this.#transformation.multiply(circle.center), radius:this.#transformation.multiply(circle.radius), style: circle.style})
    }
    return ret;
  }
  #getTransformed(){
    let transformed = {position: this.#transformation.multiply(this.#position), 
      positionCorner : this.#transformation.multiply(this.#positionCorner)}
    
    transformed.inputCircles = this.#getTransformedCircles(this.#inputCircles, transformed);
    transformed.outputCircles = this.#getTransformedCircles(this.#outputCircles, transformed);
    return transformed;
  }

  #drawCircles(circles){
    for (const circle of circles){
      context.beginPath();
      context.arc(circle.center.x, circle.center.y, circle.radius.x-circle.center.x ,0,Math.PI*2,false);
      context.fillStyle= circle.style;
      context.fill();
    }
  }

  draw(context){

    var transformed = this.#getTransformed();
    context.fillRect(transformed.position.x,transformed.position.y,transformed.positionCorner.x-transformed.position.x,transformed.positionCorner.y-transformed.position.y);
    context.fillStyle = 'blue';
    context.font = ((transformed.positionCorner.x-transformed.position.x)/10)+"px seif";
    context.fillText(this.nodeName,transformed.position.x, transformed.position.y);
    this.#drawCircles(transformed.inputCircles);
    this.#drawCircles(transformed.outputCircles);

  }

  transform(transformation){
    this.#transformation = this.#transformation.multiply(transformation);
  }

  set transformation(transformation){
    this.#transformation = transformation;
  }
  
}


class Graph{
  #nodes = [];
  #scale = 1;
  #transformation = Mat3.identity();

  construct(){

  }

  addNode(node){
    node.transformation = this.#transformation;
    this.#nodes.push(node);

  }

  draw(context){

    for (const node of this.#nodes){
      node.draw(context);
    }
  }

  transform(transformation){
    //this.#transformation = transformation.multiply(this.#transformation);
    this.#transformation = this.#transformation.multiply(transformation);
    for (let node of this.#nodes){
      node.transformation = this.#transformation;
    }
  }


  getTransformedMouse(mousePos){
    return this.#transformation.inverse().multiply(mousePos);
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







Mat3.test();

graph = new Graph();
graph.addNode(new Node([],"asd", Vec2(0,0)));
graph.addNode(new Node([],"asd2", Vec2(100,100)));

let canvas = document.getElementById("canvas");

let context = canvas.getContext("2d");

canvas.style.background = "#ff8";

canvas.width = 600;
canvas.height = 600;









let pos = Vec2(0,0);
let pos2 = Vec2(100,100);
let mousePos = Vec2(0,0);




let scale = 1;
let originx = 0;
let originy = 0;
let visibleWidth = canvas.width;
let visibleHeight = canvas.height;

fetch("/process",{method:"POST"}).then(response=>{});

var image = new Image();
image.src = "images/output_0.png";



function animate(){
    //originalX += dx;
    //originalY += dy;


    //requestAnimationFrame(animate);
    context.save();
    context.setTransform(1,0,0,1,0,0);
    context.clearRect(0,0,canvas.width,canvas.height);
    context.restore();


    
    context.drawImage(image,0,0,canvas.width,canvas.height);
    graph.draw(context);
   
    let newPos = graph.transformation.multiply(pos);
    let newPos2 = graph.transformation.multiply(pos2);
    context.fillRect(newPos.x,newPos.y,newPos2.x-newPos.x, newPos2.y-newPos.y);


}

animate();

let isMouseDown = false;
let previousPosition = Vec2(0,0);

function getMousePos(canvas, evt) {
    let rect = canvas.getBoundingClientRect();
    return Vec2(evt.clientX - rect.left, evt.clientY - rect.top);
  }
  canvas.addEventListener('mousedown', function(evt) {
    isMouseDown =true;

    mousePos = getMousePos(canvas, evt);
    previousPosition = mousePos;
  }, false);

  addEventListener("dblclick", (evt) => {
    mousePos = getMousePos(canvas, evt);
    //let transform = context.getTransform().invertSelf();
    let transformed = graph.getTransformedMouse(mousePos);
    //let transformed = Vec2(mousePos.x*transform.a+mousePos.y*transform.c + transform.e, mousePos.x*transform.b+mousePos.y*transform.d+transform.f);
    transformed.z = 1;
    graph.addNode(new Node([], "lol", transformed));
    animate();
  },false);
  canvas.addEventListener('mouseup', function(evt) {
    isMouseDown =false;
    mousePos = getMousePos(canvas, evt);

    image = new Image();
    image.src = "images/output_0.png";
  }, false);
  canvas.addEventListener('mousemove', function(evt) {
    
    mousePos = getMousePos(canvas, evt);
    if (isMouseDown){
        let dx = (mousePos.x - previousPosition.x);
        let dy = (mousePos.y - previousPosition.y);
        previousPosition = mousePos;
        //context.translate(dx,dy);
        originx -= dx;
        originy -= dy;
        graph.transform(Mat3.translate(dx,dy));
        animate();
    }

  }, false);
  let transform = Mat3.identity();
  let firstTime = true;
  canvas.addEventListener('wheel', function(event){
    event.preventDefault();
    // Get mouse offset.
    const mousex = event.clientX - canvas.offsetLeft;
    const mousey = event.clientY - canvas.offsetTop;
    // Normalize mouse wheel movement to +1 or -1 to avoid unusual jumps.
    const wheel = event.deltaY < 0 ? 1 : -1;
    
    mousePos = getMousePos(canvas, event);
    

    // Compute zoom factor.
    const zoom = Math.exp(wheel * 0.2);
    
    let transformed = Vec2(50,50);
    if(!firstTime){
      transformed = Vec2(25,25);
    }
    firstTime =false;

    //let transform = graph.transformation;
    //transform = transform.multiply(Mat3.translate(-transformed.x,-transformed.y));
    //transform = transform.multiply(Mat3.scale(zoom));
    //transform = transform.multiply(Mat3.translate(transformed.x,transformed.y));
    //graph.transformation = transform;
    //console.log(transform.data);
    //context.transform(...transform.data[0],...transform.data[1]);
    // Translate so the visible origin is at the context's origin.
    context.translate(-transformed.x, -transformed.y);
    //graph.transform(Mat3.translate(originx,originy));
    //let transformed =graph.getTransformedMouse(mousePos);

  
    //graph.transform(Mat3.translate(-transformed.x,-transformed.y));

  
    // Compute the new visible origin. Originally the mouse is at a
    // distance mouse/scale from the corner, we want the point under
    // the mouse to remain in the same place after the zoom, but this
    // is at mouse/new_scale away from the corner. Therefore we need to
    // shift the origin (coordinates of the corner) to account for this.
    //let transformed = graph.getTransformedMouse(mousePos);
    originx -= mousex/(scale*zoom) - mousex/scale;
    originy -= mousey/(scale*zoom) - mousey/scale;
    
    // Scale it (centered around the origin due to the translate above).
    context.scale(zoom, zoom);
    //graph.transform(Mat3.scale(zoom));
    //graph.zoom(mousePos,zoom);

    
    // Offset the visible origin to it's proper position.
    context.translate(transformed.x, transformed.y);
    console.log(context.getTransform().e);
    //graph.transform(Mat3.translate(transformed.x,transformed.y));
    //graph.transform(Mat3.translate(-originx,-originy));

    // Update scale and others.
    scale *= zoom;

    animate();
  },false);