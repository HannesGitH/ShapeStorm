#include "./helpers.hlsl"

struct Shape {
    
    float3 position;
    float4 rotation; //quaternion?
    float3 size;
    float3 colour;
    float lightness;
    int shapeType;
    int operation;
    float blendStrength;
    int numChildren;
};


float SphereDistance(float3 eye, float radius) {
    return length(eye) - radius;
}

float CubeDistance(float3 eye, float3 size) {
    float3 o = abs(eye) -size;
    float ud = length(max(o,0));
    float n = max(max(min(o.x,0),min(o.y,0)), min(o.z,0));
    return ud+n;
}
//TODO: mehr davon:
// Following distance functions from http://iquilezles.org/www/articles/distfunctions/distfunctions.htm
float TorusDistance(float3 eye, float r1, float r2)
{   
    float2 q = float2(length((eye).xz)-r1,eye.y);
    return length(q)-r2;
}

float PrismDistance(float3 eye, float2 h) {
    float3 q = abs(eye);
    return max(q.z-h.y,max(q.x*0.866025+eye.y*0.5,-eye.y)-h.x*0.5);
}


float CylinderDistance(float3 eye, float2 h) {
    float2 d = abs(float2(length((eye).xz), eye.y)) - h;
    return length(max(d,0.0)) + max(min(d.x,0),min(d. y,0));
}

float TubeDistance(float3 eye, float2 radius_height, float thiccness) {
    float2 d = abs(float2(length((eye).xz), eye.y)) - radius_height;
    return abs(length(max(d,0.0)) + max(min(d.x,0),min(d. y,0)))-thiccness;
}

float BoxFrameDistance( float3 p, float3 b, float e )
{
    p = abs(p  )-b;
    float3 q = abs(p+e)-e;
    return min(min(
        length(max(float3(p.x,q.y,q.z),0.0))+min(max(p.x,max(q.y,q.z)),0.0),
        length(max(float3(q.x,p.y,q.z),0.0))+min(max(q.x,max(p.y,q.z)),0.0)),
        length(max(float3(q.x,q.y,p.z),0.0))+min(max(q.x,max(q.y,p.z)),0.0));
}

float OctahedronDistance( float3 p, float s)
{
  p = abs(p);
  return (p.x+p.y+p.z-s)*0.57735027;
}

float LinkDistance( float3 p, float le, float r1, float r2 )
{
  float3 q = float3( p.x, max(abs(p.y)-le,0.0), p.z );
  return length(float2(length(q.xy)-r1,q.z)) - r2;
}

float GetShapeDistance(Shape shape, float3 eye) {

    eye = rotate_vector(eye,shape.rotation);
    eye -= shape.position;
   
    if (shape.shapeType == 0) {
        return SphereDistance(eye, shape.size.x);
    }
    else if (shape.shapeType == 1) {
        return CubeDistance(eye, shape.size);
    }
    else if (shape.shapeType == 2) {
        return TorusDistance(eye, shape.size.x, shape.size.y);
    }
    else if (shape.shapeType == 3) {
        return TubeDistance(eye, shape.size.xy,1);
    }
    else if (shape.shapeType == 4) {
        return BoxFrameDistance(eye, shape.size.xyz,0.3);
    }
    else if (shape.shapeType == 5) {
        return OctahedronDistance(eye, shape.size.x);
    }
    else if (shape.shapeType == 6) {
        return LinkDistance(eye, shape.size.x,shape.size.y,shape.size.z);
    }

    return 1000000;//float.inf?
}
