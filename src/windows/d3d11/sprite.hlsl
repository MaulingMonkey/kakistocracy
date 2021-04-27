Texture2D       t : register(t0);
SamplerState    s : register(s0);



struct Vertex {
    float4 position : POSITION0;
    float2 texcoord : TEXCOORD0;
};

struct VsToPs {
    float2 texcoord : TEXCOORD0;
    float4 position : SV_POSITION;
};

struct Pixel {
    float4 color : SV_TARGET0;
};



void vs(in Vertex v, out VsToPs o) {
    o.position = v.position;
    o.texcoord = v.texcoord;
}

void ps(in VsToPs v, out Pixel o) {
    o.color = t.Sample(s, v.texcoord);
}
