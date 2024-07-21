#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform int screenWidth;
uniform int screenHeight;

struct Camera
{
    vec3 camPos;
    vec3 front;
    vec3 right;
    vec3 up;
    float halfH;
    float halfW;
    vec3 leftbottom;
    int LoopNum;
};
uniform Camera camera;

struct Ray
{
    vec3 origin;
    vec3 direction;
    float hitMin;
};

struct Sphere
{
    int material;
    vec3 albedo;
    vec3 center;
    float radius;
};
uniform Sphere sphere[4];

struct Triangle
{
    vec3 v[3];
    vec3 n[3];
    vec3 uv[3];
};

uniform Triangle triangle[2];

struct hitRecord
{
    vec3 p;
    float hitMin;
    vec3 normal;
    int material;
    vec3 albedo;
};
hitRecord rec;

struct aabb
{
    vec3 minb, maxb;
};

struct LinearBVHNode
{
    vec3 minb, maxb;
    float primitives_num;
    float axis;
    float child_offset;
};

uint wseed;
float randcore(uint seed);
float rand();

float hitSphere(Sphere sphere, Ray r);
bool hitWorld(Ray r);
vec3 shading(Ray r);
bool intersectAABB(Ray r, aabb box, vec3 invDir, int dirIsNeg[3]);

uniform sampler2D historyTexture;
uniform sampler2D vertices_texture;
uniform sampler2D bvh_texture;

uniform sampler2D diffuse_texture0;
uniform sampler2D diffuse_texture1;
uniform sampler2D diffuse_texture2;
uniform sampler2D diffuse_texture3;
uniform sampler2D diffuse_texture4;
uniform int verticesNum;
uniform int nodeNum;
uniform float randOrigin;
uniform int depths;
uniform float index;

vec3 getData(sampler2D dataTexture, float index);

void main()
{
    wseed = uint(randOrigin * float(6.95857) * (TexCoords.x * TexCoords.y));

    vec3 historyColor = texture(historyTexture, TexCoords).rgb;

    vec2 offset = vec2((rand() - 0.5) / float(screenWidth), (rand() - 0.5) / float(screenHeight));
    // vec2 offset = vec2(0.0, 0.0);

    Ray ray;
    ray.origin = camera.camPos;
    ray.direction = normalize(camera.leftbottom + (2.0 * camera.halfW * (TexCoords.x + offset.x)) * camera.right +
                              (2.0 * camera.halfH * (TexCoords.y + offset.y)) * camera.up);
    ray.hitMin = 3.402823466e+38;

    vec3 color = shading(ray);
    color = mix(historyColor, color, 1.0 / float(camera.LoopNum));

    // FragColor = vec4((getData(bvh_texture, index)), 1.0);
    FragColor = vec4(color, 1.0);
}

float randcore(uint seed)
{
    seed = (seed ^ uint(61)) ^ (seed >> uint(16));
    seed *= uint(9);
    seed = seed ^ (seed >> uint(4));
    seed *= uint(0x27d4eb2d);
    wseed = seed ^ (seed >> uint(15));
    return float(wseed) * (1.0 / 4294967296.0);
}
float rand()
{
    return randcore(wseed);
}

vec3 random_in_unit_sphere()
{
    vec3 p;
    do
    {
        p = 2.0 * vec3(rand(), rand(), rand()) - vec3(1.0, 1.0, 1.0);
    } while (dot(p, p) >= 1.0);
    return p;
}

void setNormal(Ray r)
{
    bool frontFace = dot(r.direction, rec.normal) < 0.0;
    rec.normal = frontFace ? rec.normal : -rec.normal;
}

vec3 getData(sampler2D dataTexture, float index)
{
    float col = mod(index, 2048.0);
    float row = floor(index / 2048.0);
    vec2 texCoord = vec2(col / 2048.0, row / 2048.0);
    return texture(dataTexture, texCoord).rgb;
}

Triangle getTriangle(int index)
{
    Triangle tri;
    for (int i = 0; i < 3; i++)
    {
        tri.v[i] = getData(vertices_texture, float(index * 9 + i * 3));
        tri.n[i] = getData(vertices_texture, float(index * 9 + i * 3 + 3));
        tri.uv[i] = getData(vertices_texture, float(index * 9 + i * 3 + 6));
    }
    return tri;
}

LinearBVHNode getBVHNode(int index)
{
    LinearBVHNode node;
    node.minb = getData(bvh_texture, float(index * 3));
    node.maxb = getData(bvh_texture, float(index * 3 + 1));
    node.child_offset = getData(bvh_texture, float(index * 3 + 2)).x;
    node.primitives_num = getData(bvh_texture, float(index * 3 + 2)).y;
    node.axis = getData(bvh_texture, float(index * 3 + 2)).z;
    return node;
}

vec3 diffuse(vec3 normal)
{
    return normalize(normal + random_in_unit_sphere());
}

vec3 metal(vec3 normal, vec3 direction)
{
    return reflect(direction, normal);
}

float hitSphere(Sphere sphere, Ray r)
{
    vec3 oc = r.origin - sphere.center;
    float a = dot(r.direction, r.direction);
    float h = -dot(oc, r.direction);
    float c = dot(oc, oc) - sphere.radius * sphere.radius;
    float discriminant = h * h - a * c;
    if (discriminant < 0.0)
        return -1.0;
    else
    {
        float dist = (h - sqrt(discriminant)) / a;
        if (dist > 0.00001)
            return dist;
        else
        {
            float dist = (h + sqrt(discriminant)) / a;
            if (dist > 0.00001)
                return dist;
            else
                return -1.0;
        }
    }
}

float hitTriangle(Triangle triangle, Ray r)
{
    vec3 e1 = triangle.v[1] - triangle.v[0];
    vec3 e2 = triangle.v[2] - triangle.v[0];
    vec3 n = normalize(cross(e1, e2));
    if (dot(n, r.direction) == 0.0)
    {
        return -1.0;
    }
    vec3 s = (r.origin - triangle.v[0]);
    vec3 s1 = (cross(r.direction, e2));
    vec3 s2 = (cross(s, e1));
    float t = dot(s2, e2) / dot(s1, e1);
    float u = dot(s1, s) / dot(s1, e1);
    float v = dot(s2, r.direction) / dot(s1, e1);
    if (u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > 0.00001)
    {
        return t;
    }
    else
    {
        return -1.0;
    }
}

bool intersectBVH(Ray r)
{
    vec3 invDir = 1.0 / r.direction;
    bool hit = false;
    int dirIsNeg[3];
    dirIsNeg[0] = invDir.x < 0.0 ? 1 : 0;
    dirIsNeg[1] = invDir.y < 0.0 ? 1 : 0;
    dirIsNeg[2] = invDir.z < 0.0 ? 1 : 0;
    int toVisitOffset = 0, currentNodeIndex = 0;
    int nodesToVisit[64];
    Triangle tri;
    while (true)
    {
        LinearBVHNode node = getBVHNode(currentNodeIndex);
        aabb box;
        box.minb = node.minb;
        box.maxb = node.maxb;
        if (intersectAABB(r, box, invDir, dirIsNeg))
        {
            if (node.primitives_num > 0.0)
            {
                for (int i = 0; i < node.primitives_num; i++)
                {
                    Triangle tri_t = getTriangle(int(node.child_offset + i));
                    float dis_t = hitTriangle(tri_t, r);
                    if (dis_t > 0.0 && dis_t < r.hitMin)
                    {
                        r.hitMin = dis_t;
                        hit = true;
                        tri = tri_t;
                    }
                }
                if (toVisitOffset == 0)
                    break;
                currentNodeIndex = nodesToVisit[--toVisitOffset];
            }
            else
            {
                if (bool(dirIsNeg[int(node.axis)]))
                {
                    nodesToVisit[toVisitOffset++] = currentNodeIndex + 1;
                    currentNodeIndex = int(node.child_offset);
                }
                else
                {
                    nodesToVisit[toVisitOffset++] = int(node.child_offset);
                    currentNodeIndex = currentNodeIndex + 1;
                }
            }
        }
        else
        {
            if (toVisitOffset == 0)
                break;
            currentNodeIndex = nodesToVisit[--toVisitOffset];
        }
    }
    if (hit)
    {
        rec.p = r.origin + r.hitMin * r.direction;
        rec.normal = normalize(cross(tri.v[1] - tri.v[0], tri.v[2] - tri.v[0]));
        rec.material = 0;
        rec.albedo = vec3(0.5, 0.5, 1.0);
        rec.hitMin = r.hitMin;
        setNormal(r);
    }
    return hit;
}

bool hitWorld(Ray r)
{
    float dist = 3.402823466e+38;
    bool ifHitSphere = false;
    bool ifHitTriangle = false;
    int hitSphereIndex;
    int hitTriangleIndex;
    /*
        for (int i = 0; i < 4; i++)
        {
            float dis_t = hitSphere(sphere[i], r);
            if (dis_t > 0 && dis_t < dist)
            {
                dist = dis_t;
                hitSphereIndex = i;
                ifHitSphere = true;
            }
        }
        for (int i = 0; i < 2; i++)
        {
            float dis_t = hitTriangle(triangle[i], r);
            if (dis_t > 0 && dis_t < dist)
            {
                dist = dis_t;
                ifHitTriangle = true;
            }
        }
    */
    // for (int i = 0; i < verticesNum / 3; i++)
    /*
    for (int i = 0; i < 21000 / 3; i++)

    {
        Triangle tri = getTriangle(i);
        float dis_t = hitTriangle(tri, r);
        if (dis_t > 0 && dis_t < dist)
        {
            dist = dis_t;
            hitTriangleIndex = i;
            ifHitTriangle = true;
        }
    }
    */

    if (intersectBVH(r))
    {
        r.hitMin = rec.hitMin;
        return true;
    }

    if (ifHitSphere)
    {
        rec.p = r.origin + dist * r.direction;
        rec.normal = normalize(r.origin + dist * r.direction - sphere[hitSphereIndex].center);
        rec.material = sphere[hitSphereIndex].material;
        rec.albedo = sphere[hitSphereIndex].albedo;
        setNormal(r);
        return true;
    }
    else if (ifHitTriangle)
    {
        rec.p = r.origin + dist * r.direction;
        // rec.normal = normalize(cross(triangle[0].v[1] - triangle[0].v[0], triangle[0].v[2] - triangle[0].v[0]));
        rec.normal = normalize(cross(getTriangle(hitTriangleIndex).v[1] - getTriangle(hitTriangleIndex).v[0],
                                     getTriangle(hitTriangleIndex).v[2] - getTriangle(hitTriangleIndex).v[0]));
        rec.material = 0;
        rec.albedo = vec3(0.5, 0.5, 1.0);
        setNormal(r);
        return true;
    }
    else
        return false;
}

vec3 shading(Ray r)
{
    vec3 color = vec3(1.0, 1.0, 1.0);
    bool hitAnything = false;
    for (int i = 0; i < depths; i++)
    {
        if (hitWorld(r))
        {

            if (rec.material == 0)
            {
                r.direction = diffuse(rec.normal);
            }
            else if (rec.material == 1)
            {
                r.direction = metal(rec.normal, r.direction);
            }

            r.origin = rec.p;
            r.hitMin = 3.402823466e+38;
            color *= rec.albedo;
            hitAnything = true;
        }
        else
        {
            color *= vec3(1.0, 1.0, 1.0);
            break;
        }
    }
    // if (!hitAnything)
    //     color = vec3(0.0, 0.0, 0.0);
    return color;
}

vec3 getAABBb(aabb box, int i)
{
    return (i == 0) ? box.minb : box.maxb;
}

bool intersectAABB(Ray r, aabb box, vec3 invDir, int dirIsNeg[3])
{
    float tmin = (getAABBb(box, dirIsNeg[0]).x - r.origin.x) * invDir.x;
    float tmax = (getAABBb(box, 1 - dirIsNeg[0]).x - r.origin.x) * invDir.x;
    float tymin = (getAABBb(box, dirIsNeg[1]).y - r.origin.y) * invDir.y;
    float tymax = (getAABBb(box, 1 - dirIsNeg[1]).y - r.origin.y) * invDir.y;
    if ((tmin > tymax) || (tymin > tmax))
        return false;
    if (tymin > tmin)
        tmin = tymin;
    if (tymax < tmax)
        tmax = tymax;
    float tzmin = (getAABBb(box, dirIsNeg[2]).z - r.origin.z) * invDir.z;
    float tzmax = (getAABBb(box, 1 - dirIsNeg[2]).z - r.origin.z) * invDir.z;
    if ((tmin > tzmax) || (tzmin > tmax))
        return false;
    if (tzmin > tmin)
        tmin = tzmin;
    if (tzmax < tmax)
        tmax = tzmax;
    return tmax > 0.0;
}
