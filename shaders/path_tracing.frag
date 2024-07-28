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
    vec3 albedo;
    vec3 center;
    float radius;
};

struct Mesh
{
    vec3 v[3];
    vec3 n[3];
    vec2 uv[3];
    vec4 texID;
};

struct Triangle
{
    vec3 v[3];
    vec3 n;
    vec3 albedo;
};

struct Rect
{
    vec3 v[4];
    vec3 n;
    vec3 albedo;
};

struct hitRecord
{
    vec3 p;
    bool frontFace;
    float hitMin;
    float constant;
    vec3 normal;
    int material;
    vec3 albedo;
    vec3 light;
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
    float shape;
    float constant;
    float material;
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
uniform bool faceCull;
uniform float index;

vec3 getData(sampler2D dataTexture, float index);

void main()
{
    wseed = uint(randOrigin * float(6.95857) * (TexCoords.x * TexCoords.y));

    vec3 historyColor = texture(historyTexture, TexCoords).rgb;

    vec2 offset = vec2((rand() - 0.5) / float(screenWidth), (rand() - 0.5) / float(screenHeight));

    Ray ray;
    ray.origin = camera.camPos;
    ray.direction = normalize(camera.leftbottom + (2.0 * camera.halfW * (TexCoords.x + offset.x)) * camera.right +
                              (2.0 * camera.halfH * (TexCoords.y + offset.y)) * camera.up);
    ray.hitMin = 3.402823466e+38;

    vec3 color = shading(ray);
    color = mix(historyColor, color, 1.0 / float(camera.LoopNum));

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

float reflectance(float cosine, float ref_idx)
{
    float r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    return r0 + (1.0 - r0) * pow((1.0 - cosine), 5.0);
}

void setNormal(Ray r)
{
    bool frontFace = dot(r.direction, rec.normal) < 0.0;
    rec.frontFace = frontFace;
    rec.normal = frontFace ? rec.normal : -rec.normal;
}

vec3 getData(sampler2D dataTexture, float index)
{
    float col = mod(index + 0.5, textureSize(dataTexture, 0).x);
    float row = floor((index + 0.5) / textureSize(dataTexture, 0).y);
    vec2 texCoord = vec2(col / textureSize(dataTexture, 0).x, row / textureSize(dataTexture, 0).y);
    return texture(dataTexture, texCoord).rgb;
}

Sphere getSphere(int index)
{
    Sphere sphere;
    sphere.center = getData(vertices_texture, float(index * 11));
    sphere.albedo = getData(vertices_texture, float(index * 11 + 1));
    sphere.radius = getData(vertices_texture, float(index * 11 + 2)).x;

    return sphere;
}

Mesh getMesh(int index)
{
    Mesh mesh;
    for (int i = 0; i < 3; i++)
    {
        mesh.v[i] = getData(vertices_texture, float(index * 11 + i * 3));
        mesh.n[i] = getData(vertices_texture, float(index * 11 + i * 3 + 1));
        mesh.uv[i] = getData(vertices_texture, float(index * 11 + i * 3 + 2)).xy;
    }
    mesh.texID =
        vec4(getData(vertices_texture, float(index * 11 + 9)), getData(vertices_texture, float(index * 11 + 10)).x);
    return mesh;
}

Triangle getTriangle(int index)
{
    Triangle tri;
    for (int i = 0; i < 3; i++)
    {
        tri.v[i] = getData(vertices_texture, float(index * 11 + i));
    }
    tri.n = getData(vertices_texture, float(index * 11 + 3));
    tri.albedo = getData(vertices_texture, float(index * 11 + 4));
    return tri;
}

Rect getRect(int index)
{
    Rect rect;
    for (int i = 0; i < 4; i++)
    {
        rect.v[i] = getData(vertices_texture, float(index * 11 + i));
    }
    rect.n = getData(vertices_texture, float(index * 11 + 4));
    rect.albedo = getData(vertices_texture, float(index * 11 + 5));
    return rect;
}

LinearBVHNode getBVHNode(int index)
{
    LinearBVHNode node;
    node.minb = getData(bvh_texture, float(index * 4));
    node.maxb = getData(bvh_texture, float(index * 4 + 1));
    node.child_offset = getData(bvh_texture, float(index * 4 + 2)).x;
    node.primitives_num = getData(bvh_texture, float(index * 4 + 2)).y;
    node.axis = getData(bvh_texture, float(index * 4 + 2)).z;
    node.shape = getData(bvh_texture, float(index * 4 + 3)).x;
    node.constant = getData(bvh_texture, float(index * 4 + 3)).y;
    node.material = getData(bvh_texture, float(index * 4 + 3)).z;
    return node;
}

vec3 diffuse()
{
    vec3 out_dir = rec.normal + random_in_unit_sphere();
    if (abs(out_dir.x) < 1E-8 || abs(out_dir.y) < 1E-8 || abs(out_dir.z) < 1E-8)
    {
        out_dir = rec.normal;
    }
    return normalize(out_dir);
}

vec3 metal(vec3 direction)
{
    return reflect(direction, rec.normal);
}

vec3 dielectric(vec3 direction)
{
    float refraction_ratio = rec.frontFace ? 1.0 / rec.constant : rec.constant;
    float cos_theta = dot(-direction, rec.normal);
    float sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    bool cannot_refract = refraction_ratio * sin_theta > 1.0;
    vec3 out_dir;
    if (cannot_refract || reflectance(cos_theta, refraction_ratio) > rand())
    {
        out_dir = reflect(direction, rec.normal);
    }
    else
    {
        out_dir = refract(direction, rec.normal, refraction_ratio);
    }
    return out_dir;
}

vec3 diffuse_light(vec3 normal)
{
    return vec3(0.0, 0.0, 0.0);
}

vec3 centroidCoordinates(vec3 v0, vec3 v1, vec3 v2, vec3 p)
{
    vec3 v0v1 = v1 - v0;
    vec3 v0v2 = v2 - v0;
    vec3 v0p = p - v0;
    float d00 = dot(v0v1, v0v1);
    float d01 = dot(v0v1, v0v2);
    float d11 = dot(v0v2, v0v2);
    float d20 = dot(v0p, v0v1);
    float d21 = dot(v0p, v0v2);
    float denom = d00 * d11 - d01 * d01;
    float v = (d11 * d20 - d01 * d21) / denom;
    float w = (d00 * d21 - d01 * d20) / denom;
    float u = 1.0 - v - w;
    return vec3(u, v, w);
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
            if (faceCull)
            {
                return -1.0;
            }
            float dist = (h + sqrt(discriminant)) / a;
            if (dist > 0.00001)
                return dist;
            else
                return -1.0;
        }
    }
}

float hitMesh(Mesh mesh, Ray r)
{
    vec3 e1 = mesh.v[1] - mesh.v[0];
    vec3 e2 = mesh.v[2] - mesh.v[0];
    vec3 n = normalize(cross(e1, e2));
    if (faceCull)
    {
        if (dot(n, r.direction) >= 0.0)
        {
            return -1.0;
        }
    }
    else
    {
        if (abs(dot(n, r.direction)) < 0.00001)
        {
            return -1.0;
        }
    }
    vec3 s = (r.origin - mesh.v[0]);
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

float hitTriangle(Triangle tri, Ray r)
{

    vec3 n = normalize(tri.n);
    if (faceCull)
    {
        if (dot(n, r.direction) >= 0.0)
        {
            return -1.0;
        }
    }
    else
    {
        if (abs(dot(n, r.direction)) < 0.00001)
        {
            return -1.0;
        }
    }
    vec3 e1 = tri.v[1] - tri.v[0];
    vec3 e2 = tri.v[2] - tri.v[0];
    vec3 s = (r.origin - tri.v[0]);
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

float hitRect(Rect rect, Ray r)
{
    vec3 n = normalize(rect.n);
    if (faceCull)
    {
        if (dot(n, r.direction) >= 0.0)
        {
            return -1.0;
        }
    }
    else
    {
        if (abs(dot(n, r.direction)) < 0.00001)
        {
            return -1.0;
        }
    }
    Triangle tri1;
    Triangle tri2;
    tri1.v[0] = rect.v[0];
    tri1.v[1] = rect.v[1];
    tri1.v[2] = rect.v[2];
    tri1.n = rect.n;
    tri2.v[0] = rect.v[0];
    tri2.v[1] = rect.v[2];
    tri2.v[2] = rect.v[3];
    tri2.n = rect.n;
    float t1 = hitTriangle(tri1, r);
    float t2 = hitTriangle(tri2, r);
    if (t1 > 0.0)
    {
        return t1;
    }
    else if (t2 > 0.0)
    {
        return t2;
    }
    else
    {
        return -1.0;
    }
}

void selectMaterial(LinearBVHNode node, int material)
{
    switch (material)
    {
    case 1:
        rec.material = 1;
        break;
    case 2:
        rec.material = 2;
        break;
    case 3:
        rec.material = 3;
        rec.constant = node.constant;
        break;
    case 4:
        rec.material = 4;
        break;
    default:
        rec.material = 0;
        break;
    }
}

bool intersectBVH(Ray r)
{
    vec3 invDir = 1.0 / r.direction;
    bool hit = false;
    int hitShape = 0;
    int dirIsNeg[3];
    dirIsNeg[0] = invDir.x < 0.0 ? 1 : 0;
    dirIsNeg[1] = invDir.y < 0.0 ? 1 : 0;
    dirIsNeg[2] = invDir.z < 0.0 ? 1 : 0;
    int toVisitOffset = 0, currentNodeIndex = 0;
    int nodesToVisit[64];
    Mesh mesh;
    Sphere sphere;
    Triangle tri;
    Rect rect;
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
                    float dis_t;
                    switch (int(node.shape))
                    {
                    case 0:
                        break;
                    case 1:
                        Sphere sphere_t = getSphere(int(node.child_offset + i));
                        dis_t = hitSphere(sphere_t, r);
                        if (dis_t > 0.0 && dis_t < r.hitMin)
                        {
                            r.hitMin = dis_t;
                            hit = true;
                            sphere = sphere_t;
                            hitShape = 1;
                            selectMaterial(node, int(node.material));
                        }
                        break;
                    case 2:
                        Mesh mesh_t = getMesh(int(node.child_offset + i));
                        dis_t = hitMesh(mesh_t, r);
                        if (dis_t > 0.0 && dis_t < r.hitMin)
                        {
                            r.hitMin = dis_t;
                            hit = true;
                            mesh = mesh_t;
                            hitShape = 2;
                            selectMaterial(node, int(node.material));
                        }
                        break;
                    case 3:
                        Triangle tri_t = getTriangle(int(node.child_offset + i));
                        dis_t = hitTriangle(tri_t, r);
                        if (dis_t > 0.0 && dis_t < r.hitMin)
                        {
                            r.hitMin = dis_t;
                            hit = true;
                            tri = tri_t;
                            hitShape = 3;
                            selectMaterial(node, int(node.material));
                        }
                        break;
                    case 4:
                        Rect rect_t = getRect(int(node.child_offset + i));
                        dis_t = hitRect(rect_t, r);
                        if (dis_t > 0.0 && dis_t < r.hitMin)
                        {
                            r.hitMin = dis_t;
                            hit = true;
                            rect = rect_t;
                            hitShape = 4;
                            selectMaterial(node, int(node.material));
                        }
                        break;
                    default:
                        break;
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
        switch (hitShape)
        {
        case 1:
            rec.p = r.origin + r.hitMin * r.direction;
            rec.normal = normalize(rec.p - sphere.center);
            rec.albedo = sphere.albedo;
            rec.hitMin = r.hitMin;
            setNormal(r);
            break;
        case 2:
            rec.p = r.origin + r.hitMin * r.direction;
            vec3 centroidC = centroidCoordinates(mesh.v[0], mesh.v[1], mesh.v[2], rec.p);
            rec.normal = normalize(centroidC.x * mesh.n[0] + centroidC.y * mesh.n[1] + centroidC.z * mesh.n[2]);
            vec2 uv = centroidC.x * mesh.uv[0] + centroidC.y * mesh.uv[1] + centroidC.z * mesh.uv[2];
            switch (int(mesh.texID.x))
            {
            case 0:
                rec.albedo = texture(diffuse_texture0, uv).rgb;
                break;
            case 1:
                rec.albedo = texture(diffuse_texture1, uv).rgb;
                break;
            case 2:
                rec.albedo = texture(diffuse_texture2, uv).rgb;
                break;
            case 3:
                rec.albedo = texture(diffuse_texture3, uv).rgb;
                break;
            case 4:
                rec.albedo = texture(diffuse_texture4, uv).rgb;
                break;
            default:
                rec.albedo = vec3(0.5, 0.5, 1.0);
                break;
            }
            rec.hitMin = r.hitMin;
            setNormal(r);
            break;
        case 3:
            rec.p = r.origin + r.hitMin * r.direction;
            rec.normal = normalize(tri.n);
            rec.albedo = tri.albedo;
            rec.hitMin = r.hitMin;
            setNormal(r);
            break;
        case 4:
            rec.p = r.origin + r.hitMin * r.direction;
            rec.normal = normalize(rect.n);
            rec.albedo = rect.albedo;
            rec.hitMin = r.hitMin;
            setNormal(r);
            break;
        default:
            break;
        }
    }
    return hit;
}

bool hitWorld(Ray r)
{
    if (intersectBVH(r))
    {
        r.hitMin = rec.hitMin;
        return true;
    }

    return false;
}

vec3 shading(Ray r)
{
    vec3 color = vec3(0.0, 0.0, 0.0);
    vec3 attenuation = vec3(1.0, 1.0, 1.0);
    vec3 direction = vec3(0.0, 0.0, 0.0);
    bool hit = false;
    bool light = false;
    for (int i = 0; i < depths; i++)
    {
        if (hitWorld(r) && !light)
        {
            switch (rec.material)
            {
            case 1:
                r.direction = diffuse();
                rec.light = vec3(0.0, 0.0, 0.0);
                break;

            case 2:
                r.direction = metal(r.direction);
                rec.light = vec3(0.0, 0.0, 0.0);
                break;
            case 3:
                r.direction = dielectric(r.direction);
                rec.light = vec3(0.0, 0.0, 0.0);
                break;
            case 4:
                direction = diffuse_light(rec.normal);
                if (direction == vec3(0.0, 0.0, 0.0))
                {
                    rec.light = rec.albedo;
                    light = true;
                    break;
                }
                rec.light = vec3(0.0, 0.0, 0.0);
                r.direction = direction;
                break;
            }

            r.origin = rec.p;
            r.hitMin = 3.402823466e+38;
            hit = true;
            color += rec.light;
            if (!light)
                attenuation *= rec.albedo;
            color *= attenuation;
        }
        else
        {
            break;
        }
    }
    if (!hit)
    {
        color *= vec3(0.0, 0.0, 0.0);
    }
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
