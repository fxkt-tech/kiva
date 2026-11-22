using System;
using System.Collections;
using UnityEngine;
using UnityEngine.UI;

// All hinges use coordinates measured from the top left of the paper cutout.
public sealed class PaperTheatre : MonoBehaviour
{
    RectTransform head, sleeve, forearm, map;
    Button story;
    float reach;
    bool opening;
    GameObject task;
    Button depart;

    static RectTransform Cut(Transform parent, string asset, float x, float y,
        float width, float height, float hingeX = 0, float hingeY = 0)
    {
        var paper = new GameObject(asset, typeof(RectTransform), typeof(RawImage));
        var rect = (RectTransform)paper.transform;
        rect.SetParent(parent, false);
        rect.anchorMin = rect.anchorMax = Vector2.up;
        rect.pivot = new Vector2(hingeX / width, 1 - hingeY / height);
        rect.sizeDelta = new Vector2(width, height);
        rect.anchoredPosition = new Vector2(x, -y);
        var image = paper.GetComponent<RawImage>();
        image.texture = Resources.Load<Texture2D>("PaperTheatre/" + asset);
        image.raycastTarget = false;
        return rect;
    }

    public void Prepare(Campaign campaign,string stageTitle)
    {
        story = campaign.mainStory;
        var lobby = campaign.lobby.transform;
        // Keep the chapter information along the right edge of the stage.
        foreach (RectTransform child in lobby)
            child.anchoredPosition += new Vector2(680, 0);
        var cabin = Cut(lobby, "Cabin", 0, 0, 1280, 720);
        cabin.SetAsFirstSibling();
        var robe = Cut(lobby, "Robe", 205, 302, 260, 290);
        head = Cut(robe, "Head", 129, 25, 155, 173, 84, 146);
        map = Cut(lobby, "Map", 427, 429, 340, 153);
        sleeve = Cut(robe, "Sleeve", 153, 51, 125, 90, 20, 27);
        forearm = Cut(sleeve, "Forearm", 103, 66, 134, 64, 20, 27);
        robe.SetAsLastSibling();
        // The chart is behind the outstretched hand.
        Cut(campaign.chapters.transform, "Ferry", 0, 0, 1280, 720).SetAsFirstSibling();
        var stages = campaign.stages.transform;
        Cut(stages, "Ferry", 0, 0, 1280, 720).SetAsFirstSibling();
        depart = campaign.enterBattle;
        task = new GameObject("渡口任务签", typeof(RectTransform));
        var sheet = (RectTransform)task.transform;
        sheet.SetParent(stages, false);
        sheet.anchorMin = Vector2.zero;
        sheet.anchorMax = Vector2.one;
        sheet.offsetMin = sheet.offsetMax = Vector2.zero;
        for (int i = stages.childCount - 1; i >= 0; i--)
        {
            var child = stages.GetChild(i);
            if (child == sheet || child == campaign.stageBack.transform || child.GetComponent<RawImage>()) continue;
            child.SetParent(sheet, false);
            child.SetAsFirstSibling();
        }
        depart.GetComponentInChildren<Text>().text = "出发   →";
        var flag = Instantiate(depart, stages, false);
        flag.name = "渡口旗";
        var flagRect = (RectTransform)flag.transform;
        flagRect.anchoredPosition = new Vector2(824, -365);
        flagRect.sizeDelta = new Vector2(155, 52);
        var label = flag.GetComponentInChildren<Text>();
        label.rectTransform.sizeDelta = flagRect.sizeDelta;
        label.text = "1-1   "+stageTitle;
        flag.onClick.AddListener(() => task.SetActive(true));
    }

    public void ResetStage() => task.SetActive(false);

    public void OpenChart(Action onOpened)
    {
        if (!opening) StartCoroutine(Unfold(onOpened));
    }

    IEnumerator Unfold(Action onOpened)
    {
        opening = true;
        story.interactable = false;
        float elapsed = 0;
        while (elapsed < .85f)
        {
            elapsed += Time.unscaledDeltaTime;
            reach = Mathf.SmoothStep(0, 1, elapsed / .45f);
            float unfold = Mathf.SmoothStep(0, 1, (elapsed - .34f) / .51f);
            map.localScale = new Vector3(1 + unfold * 2, 1 + unfold * 3, 1);
            map.anchoredPosition = Vector2.Lerp(new Vector2(427,-429),new Vector2(125,-190),unfold);
            yield return null;
        }
        onOpened();
        reach = 0;
        map.localScale = Vector3.one;
        map.anchoredPosition = new Vector2(427,-429);
        story.interactable = true;
        opening = false;
    }

    void Update()
    {
        if (!head || !head.gameObject.activeInHierarchy) return;
        float beat = Mathf.Sin(Time.unscaledTime * 1.5f);
        head.localRotation = Quaternion.Euler(0, 0, -3 - beat * 2 - reach * 8);
        sleeve.localRotation = Quaternion.Euler(0, 0, -27 + beat * 2 + reach * 12);
        forearm.localRotation = Quaternion.Euler(0, 0, 49 - beat * 4 - reach * 56);
    }
}
