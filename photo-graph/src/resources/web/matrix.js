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
        //3x2 matrix multiplication
        // let a0 = factor.data[0][0],
        //   a1 = factor.data[1][0],
        //   a2 = factor.data[0][1],
        //   a3 = factor.data[1][1],
        //   a4 =factor.data[0][2],
        //   a5 = factor.data[1][2];
        // let b0 = this.data[0][0],
        //   b1 = this.data[1][0],
        //   b2 = this.data[0][1],
        //   b3 = this.data[1][1],
        //   b4 = this.data[0][2],
        //   b5 = this.data[1][2];
        // result.data[0][0] = a0 * b0 + a2 * b1;
        // result.data[1][0] = a1 * b0 + a3 * b1;
        // result.data[0][1] = a0 * b2 + a2 * b3;
        // result.data[1][1] = a1 * b2 + a3 * b3;
        // result.data[0][2] = a0 * b4 + a2 * b5 + a4;
        // result.data[1][2] = a1 * b4 + a3 * b5 + a5;
        // return result;
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

    distance(other){
      return Math.sqrt(Math.pow(other.x-this.x,2)+Math.pow(other.y-this.y,2));
    }
  }
  
  function Vec2(x,y){
    return new Vec3(x,y,1);
  }
  