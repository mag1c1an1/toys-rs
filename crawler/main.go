package main

import (
	"fmt"
	"io"
	"log"
	"net/http"
	"net/url"
	"os"
	"strings"

	"golang.org/x/net/html"
)

// 解析 HTML 文档，提取所有 PDF 链接
func extractPDFLinks(baseURL string) ([]string, error) {
	resp, err := http.Get(baseURL)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("failed to fetch URL: %s, status code: %d", baseURL, resp.StatusCode)
	}

	doc, err := html.Parse(resp.Body)
	if err != nil {
		return nil, err
	}

	var pdfLinks []string
	var f func(*html.Node)
	f = func(n *html.Node) {
		if n.Type == html.ElementNode && n.Data == "a" {
			for _, a := range n.Attr {
				if a.Key == "href" && strings.HasSuffix(a.Val, "up.pdf") {
					// 将相对链接转换为绝对链接
					absoluteURL, err := resolveURL(baseURL, a.Val)
					if err == nil {
						pdfLinks = append(pdfLinks, absoluteURL)
					}
				}
			}
		}
		for c := n.FirstChild; c != nil; c = c.NextSibling {
			f(c)
		}
	}
	f(doc)

	return pdfLinks, nil
}

// 解析相对链接为绝对链接
func resolveURL(base string, relative string) (string, error) {
	baseURL, err := url.Parse(base)
	if err != nil {
		return "", err
	}
	relativeURL, err := url.Parse(relative)
	if err != nil {
		return "", err
	}
	return baseURL.ResolveReference(relativeURL).String(), nil
}

// 下载 PDF 文件
func downloadFile(url string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("failed to download file: %s, status code: %d", url, resp.StatusCode)
	}

	// 创建文件
	fileName := strings.Split(url, "/")
	out, err := os.Create(fileName[len(fileName)-1])
	if err != nil {
		return err
	}
	defer out.Close()

	// 将响应体写入文件
	_, err = io.Copy(out, resp.Body)
	return err
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: go run main.go <url>")
		return
	}

	url := os.Args[1]
	pdfLinks, err := extractPDFLinks(url)
	if err != nil {
		log.Fatalf("Error: %v", err)
	}

	fmt.Printf("PDF links found in %s:\n", url)
	for _, link := range pdfLinks {
		fmt.Println(link)
		if err := downloadFile(link); err != nil {
			log.Printf("Error downloading PDF: %v\n", err)
		} else {
			fmt.Printf("Downloaded: %s\n", link)
		}
	}
}
