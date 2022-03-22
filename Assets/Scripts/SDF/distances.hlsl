#include "./primitives.hlsl"

extern float maxDst;
extern StructuredBuffer<Shape> shapes;
extern int numShapes;

// polynomial smooth min (k = 0.1);
// from https://www.iquilezles.org/www/articles/smin/smin.htm
float Blend( float a, float b, float k )
{
    float h = clamp( 0.5+0.5*(b-a)/k, 0.0, 1.0 );
    return lerp( b, a, h ) - k*h*(1.0-h);
}

float Combine(float dstA, float dstB, int operation, float blendStrength) {
    float dst = dstA;

    if (operation == 0) {
        if (dstB < dstA) {
            dst = dstB;
        }
    } 
    // Blend
    else if (operation == 1) {
        float blend = Blend(dstA,dstB,blendStrength);
        dst = blend;
    }
    // Cut
    else if (operation == 2) {
        // max(a,-b)
        if (-dstB > dst) {
            dst = -dstB;
        }
    }
    // Mask
    else if (operation == 3) {
        // max(a,b)
        if (dstB > dst) {
            dst = dstB;
        }
    }
    return dst;
}
float Distance(float3 eye) {
    // return 1;
    float globalDst = maxDst;
    
    for (int i = 0; i < numShapes; i ++) {
        Shape shape = shapes[i];
        int numChildren = shape.numChildren;

        float localDst = GetShapeDistance(shape,eye);


        for (int j = 0; j < numChildren; j ++) {
            Shape childShape = shapes[i+j+1];
            float childDst = GetShapeDistance(childShape,eye);

            float combined = Combine(localDst, childDst, childShape.operation, childShape.blendStrength);
            localDst = combined;
        }
        i+=numChildren; // skip over children in outer loop
        
        float globalCombined = Combine(globalDst, localDst, shape.operation, shape.blendStrength);
        globalDst = globalCombined;        
    }

    return globalDst;
}