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


float SphereDistance(float3 eye, float3 centre, float radius) {
    return distance(eye, centre) - radius;
}

float CubeDistance(float3 eye, float3 centre, float3 size) {
    float3 o = abs(eye-centre) -size;
    float ud = length(max(o,0));
    float n = max(max(min(o.x,0),min(o.y,0)), min(o.z,0));
    return ud+n;
}
//TODO: mehr davon:
// Following distance functions from http://iquilezles.org/www/articles/distfunctions/distfunctions.htm
float TorusDistance(float3 eye, float3 centre, float r1, float r2)
{   
    float2 q = float2(length((eye-centre).xz)-r1,eye.y-centre.y);
    return length(q)-r2;
}

float PrismDistance(float3 eye, float3 centre, float2 h) {
    float3 q = abs(eye-centre);
    return max(q.z-h.y,max(q.x*0.866025+eye.y*0.5,-eye.y)-h.x*0.5);
}


float CylinderDistance(float3 eye, float3 centre, float2 h) {
    float2 d = abs(float2(length((eye).xz), eye.y)) - h;
    return length(max(d,0.0)) + max(min(d.x,0),min(d. y,0));
}

float TubeDistance(float3 eye, float3 centre, float2 radius_height, float4 rotation) { //TODO:roation (apply inverse to eye?)
    float2 d = abs(float2(length((eye).xz), eye.y)) - radius_height;
    return length(max(d,0.0)) + max(min(d.x,0),min(d. y,0));
}

float GetShapeDistance(Shape shape, float3 eye) {
   
    if (shape.shapeType == 0) {
        return SphereDistance(eye, shape.position, shape.size.x);
    }
    else if (shape.shapeType == 1) {
        return CubeDistance(eye, shape.position, shape.size);
    }
    else if (shape.shapeType == 2) {
        return TorusDistance(eye, shape.position, shape.size.x, shape.size.y);
    }
    else if (shape.shapeType == 3) {
        return TubeDistance(eye, shape.position, shape.size.xy, shape.rotation);
    }

    return 1000000;//float.inf?
}
